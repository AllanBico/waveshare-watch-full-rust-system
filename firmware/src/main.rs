//! Firmware entry point.
//!
//! Brings up the AXP2101, PCF85063A, FT3168, and QMI8658 on one I2C bus,
//! chooses the runtime mode from VBUS (USB -> Desk, else Watch), owns the
//! display and input devices across the awake window, and deep-sleeps. The
//! external RTC keeps wall-clock time. The pieces `main` calls live in
//! [`runtime`] and [`display`].

#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig, RtcPin};
use esp_hal::rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

use common::{Mode, MotionSample, TouchReport, WakeOnMotion, WakeReason};
use drivers::axp2101::Axp2101;
use drivers::board;
use drivers::ft3168::Ft3168;
use drivers::pcf85063a::Pcf85063a;
use drivers::qmi8658::Qmi8658;
use embedded_hal_bus::i2c::RefCellDevice;
use power::PowerManager;

mod display;
mod log_boot;
mod persist;
mod retained;
mod runtime;

use esp_alloc as _;
use esp_backtrace as _;

use runtime::{desk_keepalive, halt, open_bus, report_boot, report_time, wake_reason, watch_awake};

esp_bootloader_esp_idf::esp_app_desc!();

/// Awake time budget granted on each watch-mode wake.
const AWAKE_BUDGET_MS: u32 = 5_000;

/// How long to stay awake before sleeping in watch mode.
// TODO(phase-5): return to the 2 s watch budget once the input viewer is gone.
const AWAKE_WINDOW: Duration = Duration::from_secs(5);

/// Deep-sleep timer interval. Longer for wake-on-motion debugging.
const SLEEP_TIMER_MS: u32 = 30_000;

/// Wake-on-Motion threshold. Tune on hardware with the input viewer.
const WOM_THRESHOLD_MG: u8 = 80;
/// Accelerometer samples ignored after entering Wake-on-Motion.
const WOM_BLANKING_SAMPLES: u8 = 10;

/// SRAM heap size reserved for the runtime.
const SRAM_HEAP_BYTES: usize = 64 * 1024;

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    esp_alloc::heap_allocator!(size: SRAM_HEAP_BYTES);

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz));

    let (_, psram_bytes) = esp_hal::psram::psram_raw_parts(&peripherals.PSRAM);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    report_boot(psram_bytes);

    let raw_wake = wake_reason();

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

    // Mode is chosen from VBUS only: USB -> Desk, otherwise Watch.
    let vbus = power.is_vbus_present().unwrap_or(false);
    let mode = if vbus { Mode::Desk } else { Mode::Watch };

    // Ext1 is shared by the PWR button and the IMU INT1. Read the IMU
    // Wake-on-Motion flag before init clears it, so a motion wake is logged as
    // Motion instead of Button.
    let mut imu = Qmi8658::new(RefCellDevice::new(bus), board::IMU_I2C_ADDR, Delay::new());
    let wom = if raw_wake.is_some() {
        imu.take_wake_event().ok()
    } else {
        None
    };
    let motion_wake = matches!(raw_wake, Some(WakeReason::Button)) && wom == Some(true);
    let wake = if motion_wake {
        Some(WakeReason::Motion)
    } else {
        raw_wake
    };
    match wake {
        Some(reason) => println!("[pm] woke by {:?} (wom {:?})", reason, wom),
        None => println!("[pm] cold boot"),
    }

    if let Err(error) = power.set_mode(mode) {
        println!("[pm] set_mode failed: {}", error.to_common());
        halt().await
    }
    println!("[pm] mode {:?} (vbus: {})", mode, vbus);

    let battery_mv = power
        .pmic_mut()
        .read_battery_millivolts()
        .map(|millivolts| millivolts.value())
        .unwrap_or(0);
    let report = persist::init(peripherals.FLASH, mode, battery_mv, wake, now);
    if mode == Mode::Desk {
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
    // explicitly so the FT3168 stops scanning before deep sleep.
    let mut touch = Ft3168::new(
        RefCellDevice::new(bus),
        board::TP_I2C_ADDR,
        Output::new(peripherals.GPIO9, Level::Low, OutputConfig::default()),
        Delay::new(),
    );
    let touch_error = match touch.init() {
        Ok(()) => None,
        Err(error) => {
            if mode == Mode::Desk {
                println!("[tp] init failed: {}", error);
            }
            Some(error.to_common().code())
        }
    };
    let chip_id = touch.read_chip_id().ok();
    if mode == Mode::Desk {
        match chip_id {
            Some(id) => println!("[tp] FT3168 id 0x{:02X}", id),
            None => println!("[tp] FT3168 id unreadable"),
        }
    }

    let imu_error = match imu.init() {
        Ok(()) => None,
        Err(error) => {
            if mode == Mode::Desk {
                println!("[imu] init failed: {}", error);
            }
            Some(error.to_common().code())
        }
    };
    let imu_id = imu.read_chip_id().ok();
    let revision = imu.read_revision().ok();
    if mode == Mode::Desk {
        match (imu_id, revision) {
            (Some(id), Some(revision)) => {
                println!("[imu] QMI8658 id 0x{:02X} rev 0x{:02X}", id, revision)
            }
            _ => println!("[imu] identity unreadable"),
        }
    }

    if let Some(panel) = display.as_mut() {
        if let Err(error) = panel.init() {
            println!("[lcd] init failed: {:?}", error);
        } else {
            let touch_report = TouchReport::empty();
            let motion = MotionSample::ZERO;
            let viewer = display::Viewer {
                touch: &touch_report,
                motion: &motion,
                wake,
                wom,
                error: touch_error.or(imu_error),
            };
            if let Err(error) = panel.draw_input_viewer(&viewer) {
                println!("[lcd] viewer failed: {:?}", error);
            }
        }
    }

    if let Some(reason) = wake {
        if let Err(error) = power.request_awake(reason, AWAKE_BUDGET_MS, 0) {
            println!("[pm] awake failed: {}", error.to_common());
        }
    }

    {
        let mut input = runtime::InputState::new(
            &mut touch,
            &mut imu,
            touch_error,
            imu_error,
            mode == Mode::Desk,
        );
        input.wake = wake;
        input.wom = wom;

        if mode == Mode::Desk {
            desk_keepalive(&mut power, &mut display, &mut input).await;
            if let Err(error) = power.set_mode(Mode::Watch) {
                println!("[pm] set_mode failed: {}", error.to_common());
                halt().await
            }
            Timer::after(AWAKE_WINDOW).await;
        } else {
            watch_awake(&mut display, &mut input, AWAKE_WINDOW).await;
        }
    }

    // Leave the IMU in Wake-on-Motion so the next motion can wake the SoC.
    if imu_error.is_none() {
        let config = WakeOnMotion::new(WOM_THRESHOLD_MG, WOM_BLANKING_SAMPLES);
        if let Err(error) = imu.configure_wake_on_motion(config) {
            if mode == Mode::Desk {
                println!("[imu] wake-on-motion failed: {}", error);
            }
        }
    }
    if let Err(error) = touch.sleep() {
        if mode == Mode::Desk {
            println!("[tp] sleep failed: {}", error);
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
    let mut pwr_button = peripherals.GPIO10;
    let mut imu_int = peripherals.GPIO21;
    let mut wake_pins: [&mut dyn RtcPin; 2] = [&mut pwr_button, &mut imu_int];
    let external = Ext1WakeupSource::new(&mut wake_pins, WakeupLevel::High);
    rtc.sleep_deep(&[&timer, &external]);
}
