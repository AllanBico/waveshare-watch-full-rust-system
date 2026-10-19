//! Firmware entry point.
//!
//! Runs one boot chain for both modes: PMIC, RTC, mode selection, storage and
//! settings, then the PSRAM and peripheral hooks named by [`ModePlan`]. Watch
//! and desk never duplicate that sequence; only the plan and the awake loop
//! differ. The external RTC keeps wall-clock time. The pieces `main` calls live
//! in [`runtime`], [`boot`], [`awake`], [`present`], [`sleep_cycle`], and
//! [`display`].

#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

use common::{Mode, ModePlan, Percent, WakeReason};
use drivers::axp2101::Axp2101;
use drivers::board;
use drivers::pcf85063a::Pcf85063a;
use drivers::qmi8658::Qmi8658;
use embedded_hal_bus::i2c::RefCellDevice;
use power::{PowerManager, WakeAction};
use settings::BootPolicy;

mod alarm_boot;
mod awake;
mod boot;
mod display;
mod log_boot;
mod log_view;
mod persist;
mod present;
mod registry;
mod retained;
mod runtime;
mod session;
mod sleep_cycle;

use boot::SLEEP_TIMER_MS;

use esp_alloc as _;
use esp_backtrace as _;

use runtime::{debounce_power_button, halt, open_bus, report_boot, report_time, wake_reason};

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz));

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let raw_wake = wake_reason();
    let (mut pwr_button, button_pressed) =
        debounce_power_button(peripherals.GPIO10, raw_wake).await;

    // One boot chain: PMIC -> mode -> RTC -> storage/settings -> PSRAM ->
    // display -> mode peripherals. `ModePlan` is the only mode seam.
    let bus = open_bus(peripherals.I2C0, peripherals.GPIO15, peripherals.GPIO14).await;

    let mut pmic = match Axp2101::new(RefCellDevice::new(bus), board::PMIC_I2C_ADDR) {
        Ok(pmic) => pmic,
        Err(error) => {
            println!("[pm] AXP2101 probe failed: {}", error);
            halt().await
        }
    };
    let chip_id = pmic.read_chip_id().ok();

    let mut power = PowerManager::new(pmic, Mode::Watch);
    power.set_sleep_timer_ms(SLEEP_TIMER_MS);
    if let Err(error) = power.sync() {
        println!("[pm] rail sync failed: {}", error.to_common());
        halt().await
    }

    // Mode is chosen from VBUS only. AXP IRQ routing to an RTC GPIO is
    // unconfirmed, so a plug while asleep is picked up on the next motion,
    // button, or timer wake; while awake, `session` polls VBUS.
    let vbus = power.is_vbus_present().unwrap_or(false);
    let mode = if vbus { Mode::Desk } else { Mode::Watch };
    let plan = ModePlan::for_mode(mode);
    let log = plan.logging;
    if log {
        match chip_id {
            Some(id) => println!("[pm] AXP2101 id 0x{:02X}", id),
            None => println!("[pm] AXP2101 id unreadable"),
        }
    }

    let mut clock = Pcf85063a::new(RefCellDevice::new(bus), board::RTC_I2C_ADDR);
    report_time(&mut clock, log);
    let now = runtime::read_timestamp(&mut clock);

    // Read the IMU Wake-on-Motion flag before init clears it, so a motion wake
    // is logged as Motion instead of Button.
    let mut imu = Qmi8658::new(RefCellDevice::new(bus), board::IMU_I2C_ADDR, Delay::new());
    let wom = if raw_wake.is_some() {
        imu.take_wake_event().ok()
    } else {
        None
    };
    // A held PWR button takes precedence over a coincident WoM event.
    let motion_wake =
        matches!(raw_wake, Some(WakeReason::Button)) && wom == Some(true) && !button_pressed;
    let wake = if motion_wake {
        Some(WakeReason::Motion)
    } else {
        raw_wake
    };
    if let Err(error) = power.set_mode(mode) {
        if log {
            println!("[pm] set_mode failed: {}", error.to_common());
        }
        halt().await
    }
    if log {
        println!("[pm] mode {:?} (vbus: {})", mode, vbus);
    }

    let snapshot = power.pmic_mut().read_power_snapshot().ok();
    let battery_mv = snapshot
        .map(|snapshot| snapshot.battery.value())
        .unwrap_or(0);
    let battery_percent = snapshot
        .map(|snapshot| snapshot.percent)
        .unwrap_or(Percent::new(0));
    let is_charging = snapshot
        .map(|snapshot| snapshot.status.charging)
        .unwrap_or(false);
    let persisted = persist::init(peripherals.FLASH, mode, battery_mv, wake, now);
    let report = persisted.report;
    let mut settings = persisted.settings;
    let device = persisted.device;
    let mut alarms = persisted.alarms;
    let alarm = persisted.alarm;
    let wake = persisted.wake;
    let show_home = WakeAction::for_wake(mode, wake).presents_home();
    if log {
        match wake {
            Some(reason) => println!(
                "[pm] woke by {:?} (wom {:?}, btn {})",
                reason, wom, button_pressed
            ),
            None => println!("[pm] cold boot"),
        }
    }
    let policy = BootPolicy::from_settings(settings.settings());

    let mut psram = Some(peripherals.PSRAM);
    let psram_bytes = if plan.initialize_psram {
        if let Some(token) = psram.take() {
            boot::init_psram(token)
        } else {
            0
        }
    } else {
        0
    };
    report_boot(mode, psram_bytes, log);
    if log {
        println!("[boot] policy {:?}", policy);
        println!(
            "[kv] {} boot {} log {:?} buf {} drop {} batt {} error {:?}",
            report.label(),
            report.boot(),
            report.log(),
            report.buffered(),
            report.dropped(),
            report.battery_mv(),
            report.error_code()
        );
    }

    // Own the panel across the awake window so it can sleep before the rail cut.
    let mut display = if show_home {
        match display::Display::new(display::Peripherals {
            spi: peripherals.SPI2,
            dma: peripherals.DMA_CH0,
            sclk: peripherals.GPIO11,
            cs: peripherals.GPIO12,
            sio0: peripherals.GPIO4,
            sio1: peripherals.GPIO5,
            sio2: peripherals.GPIO6,
            sio3: peripherals.GPIO7,
            reset: peripherals.GPIO8,
        }) {
            Ok(display) => Some(display),
            Err(error) => {
                crate::retained::record_error(common::Error::HardwareFailure.code());
                if log {
                    println!("[lcd] init failed: {:?}", error);
                }
                None
            }
        }
    } else {
        None
    };
    // Touch stays powered across the awake window; reset and put it to sleep
    // explicitly so the FT3168 stops scanning before deep sleep. Watch skips it.
    let mut touch = if plan.initialize_touch {
        Some(boot::init_touch(
            RefCellDevice::new(bus),
            peripherals.GPIO9,
            plan.logging,
        ))
    } else {
        None
    };

    let imu_error = boot::init_imu(&mut imu, plan.logging);

    if let Some(panel) = display.as_mut() {
        if let Err(error) = panel.init() {
            crate::retained::record_error(common::Error::HardwareFailure.code());
            if log {
                println!("[lcd] init failed: {:?}", error);
            }
        }
    }

    if show_home {
        let strength = settings.settings().sound.haptics;
        let mut motor = if alarm.is_some() && strength.value() > 0 {
            Some(drivers::haptics::Motor::new(peripherals.GPIO18))
        } else {
            None
        };
        let haptics = motor
            .as_mut()
            .map(|motor| session::Haptics { motor, strength });
        if let Err(error) = awake::run(awake::Inputs {
            mode,
            now,
            battery: battery_percent,
            is_charging,
            wake,
            alarm,
            settings: &mut settings,
            device,
            battery_mv,
            display: &mut display,
            touch: &mut touch,
            power: &mut power,
            psram: &mut psram,
            log: plan.logging,
            haptics,
        })
        .await
        {
            crate::retained::record_error(common::Error::NotReady.code());
            if log {
                println!("[awake] failed: {:?}", error);
            }
            if let Some(motor) = motor.as_mut() {
                motor.set(false);
            }
            halt().await
        }
        if let Some(motor) = motor.as_mut() {
            motor.set(false);
        }
    }

    let sleep_now = runtime::read_timestamp(&mut clock);
    sleep_cycle::enter(sleep_cycle::SleepInputs {
        display: &mut display,
        touch: &mut touch,
        imu: &mut imu,
        imu_error,
        power: &mut power,
        settings: settings.settings(),
        pwr_button: &mut pwr_button,
        alarms: &mut alarms,
        now: sleep_now,
        imu_int: peripherals.GPIO21,
        lpw: peripherals.LPWR,
        log,
    })
    .await
}
