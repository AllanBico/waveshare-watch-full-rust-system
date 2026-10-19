//! Firmware entry point.
//!
//! Runs one boot chain for both modes: PMIC, RTC, mode selection, storage and
//! settings, then the PSRAM and peripheral hooks named by [`boot::Plan`]. Watch
//! and desk never duplicate that sequence; only the plan and the awake loop
//! differ. The external RTC keeps wall-clock time. The pieces `main` calls live
//! in [`runtime`], [`boot`], [`present`], and [`display`].

#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Pull, RtcPin, RtcPinWithResistors};
use esp_hal::rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

use common::{Mode, Percent, WakeOnMotion, WakeReason};
use drivers::axp2101::Axp2101;
use drivers::board;
use drivers::pcf85063a::Pcf85063a;
use drivers::qmi8658::Qmi8658;
use embedded_hal_bus::i2c::RefCellDevice;
use power::PowerManager;
use settings::BootPolicy;

mod boot;
mod display;
mod log_boot;
mod persist;
mod present;
mod registry;
mod retained;
mod runtime;
mod session;

use boot::{SLEEP_TIMER_MS, SRAM_HEAP_BYTES, VBUS_POLL_PERIOD, WOM_BLANKING_SAMPLES};

use esp_alloc as _;
use esp_backtrace as _;

use runtime::{debounce_button, halt, open_bus, report_boot, report_time, wake_reason};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    esp_alloc::heap_allocator!(size: SRAM_HEAP_BYTES);

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::_80MHz));

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    let raw_wake = wake_reason();
    let mut pwr_button = peripherals.GPIO10;
    let button_pressed = if matches!(raw_wake, Some(WakeReason::Button)) {
        let input = Input::new(
            pwr_button.reborrow(),
            InputConfig::default().with_pull(Pull::Up),
        );
        debounce_button(&input).await
    } else {
        false
    };

    // One boot chain: PMIC -> RTC -> mode -> storage/settings -> PSRAM ->
    // display -> mode peripherals. `Plan` is the only mode seam.
    let bus = open_bus(peripherals.I2C0, peripherals.GPIO15, peripherals.GPIO14).await;

    let mut pmic = match Axp2101::new(RefCellDevice::new(bus), board::PMIC_I2C_ADDR) {
        Ok(pmic) => pmic,
        Err(error) => {
            println!("[pm] AXP2101 probe failed: {}", error);
            halt().await
        }
    };
    println!("[pm] AXP2101 id 0x{:02X}", pmic.read_chip_id().unwrap_or(0));

    let mut clock = Pcf85063a::new(RefCellDevice::new(bus), board::RTC_I2C_ADDR);
    report_time(&mut clock);
    let now = runtime::read_timestamp(&mut clock);

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
    match wake {
        Some(reason) => println!(
            "[pm] woke by {:?} (wom {:?}, btn {})",
            reason, wom, button_pressed
        ),
        None => println!("[pm] cold boot"),
    }

    if let Err(error) = power.set_mode(mode) {
        println!("[pm] set_mode failed: {}", error.to_common());
        halt().await
    }
    println!("[pm] mode {:?} (vbus: {})", mode, vbus);

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
    let settings = persisted.settings;
    let policy = BootPolicy::from_settings(settings.settings());

    let plan = boot::Plan::for_mode(mode);
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
    report_boot(mode, psram_bytes);
    if plan.logging {
        println!("[boot] policy {:?}", policy);
    }

    if plan.logging {
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
    let mut display = match display::Display::new(display::Peripherals {
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
            println!("[lcd] init failed: {:?}", error);
            None
        }
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
            println!("[lcd] init failed: {:?}", error);
        }
    }

    if let Some(reason) = wake {
        if let Err(error) = power.request_awake(reason, policy.awake_budget_ms, 0) {
            println!("[pm] awake failed: {}", error.to_common());
        }
    }

    let mut shell = match session::boot(
        mode,
        now,
        battery_percent,
        is_charging,
        settings.settings().display.time_format,
    ) {
        Ok(shell) => shell,
        Err(error) => {
            println!("[shell] boot failed: {:?}", error);
            halt().await
        }
    };
    if let Some(panel) = display.as_mut() {
        if let Err(error) = session::run(
            panel,
            &mut shell,
            &mut power,
            settings.settings(),
            &mut psram,
            plan.logging,
        )
        .await
        {
            println!("[session] failed: {:?}", error);
            halt().await
        }
    } else if mode == Mode::Desk {
        while power.is_vbus_present().unwrap_or(true) {
            Timer::after(VBUS_POLL_PERIOD).await;
        }
        if let Err(error) = power.set_mode(Mode::Watch) {
            println!("[pm] set_mode failed: {}", error.to_common());
            halt().await
        }
    } else {
        Timer::after(Duration::from_millis(u64::from(policy.awake_budget_ms))).await;
    }

    // Re-derive after any live switch so pre-sleep wake settings are current.
    let policy = BootPolicy::from_settings(settings.settings());
    // Arm Wake-on-Motion only when the derived policy allows it.
    if imu_error.is_none() && policy.wake_on_motion {
        let config = WakeOnMotion::new(policy.motion_threshold_mg, WOM_BLANKING_SAMPLES);
        if let Err(error) = imu.configure_wake_on_motion(config) {
            if plan.logging {
                println!("[imu] wake-on-motion failed: {}", error);
            }
        }
    }
    if let Some(touch) = touch.as_mut() {
        if let Err(error) = touch.sleep() {
            if plan.logging {
                println!("[tp] sleep failed: {}", error);
            }
        }
    }

    // Sleep the panel before `power` cuts its enable rail (ALDO2).
    if let Some(display) = display.as_mut() {
        if let Err(error) = display.sleep() {
            println!("[lcd] sleep failed: {:?}", error);
        }
    }

    let plan = match power.plan_sleep() {
        Ok(plan) => plan,
        Err(error) => {
            println!("[pm] plan_sleep failed: {}", error.to_common());
            halt().await
        }
    };
    println!("[pm] sleeping {} ms", plan.timer_ms);

    let mut rtc = Rtc::new(peripherals.LPWR);
    let timer = TimerWakeupSource::new(core::time::Duration::from_millis(u64::from(plan.timer_ms)));
    // The PWR button needs the RTC pull-up: the external network alone did not
    // produce a reliable high when tested with the digital input.
    pwr_button.rtcio_pullup(true);
    pwr_button.rtcio_pulldown(false);
    let mut imu_int = peripherals.GPIO21;
    let mut wake_pins: [&mut dyn RtcPin; 2] = [&mut pwr_button, &mut imu_int];
    let external = Ext1WakeupSource::new(&mut wake_pins, WakeupLevel::High);
    rtc.sleep_deep(&[&timer, &external]);
}
