//! Firmware entry point.
//!
//! Brings up the AXP2101 and the PCF85063A on one I2C bus, chooses the runtime
//! mode from VBUS (USB -> Desk, else Watch), owns the display across the awake
//! window, and deep-sleeps. The external RTC keeps wall-clock time. The pieces
//! `main` calls live in [`runtime`] and [`display`].

#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::RtcPin;
use esp_hal::rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel};
use esp_hal::rtc_cntl::Rtc;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

use common::Mode;
use drivers::axp2101::Axp2101;
use drivers::board;
use drivers::pcf85063a::Pcf85063a;
use embedded_hal_bus::i2c::RefCellDevice;
use power::PowerManager;

mod display;
mod persist;
mod runtime;

use esp_alloc as _;
use esp_backtrace as _;

#[cfg(not(feature = "trace"))]
use runtime::refresh;
use runtime::{desk_keepalive, halt, open_bus, report_boot, report_time, wake_reason};

esp_bootloader_esp_idf::esp_app_desc!();

/// Awake time budget granted on each watch-mode wake.
const AWAKE_BUDGET_MS: u32 = 5_000;

/// How long to stay awake before sleeping in watch mode.
#[cfg(feature = "trace")]
const AWAKE_WINDOW: Duration = Duration::from_secs(5);
/// How long to stay awake before sleeping in watch mode.
#[cfg(not(feature = "trace"))]
const AWAKE_WINDOW: Duration = Duration::from_secs(2);

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

    let wake = wake_reason();
    match wake {
        Some(reason) => println!("[pm] woke by {:?}", reason),
        None => println!("[pm] cold boot"),
    }

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

    let mut power = PowerManager::new(pmic, Mode::Watch);
    if let Err(error) = power.sync() {
        println!("[pm] rail sync failed: {}", error.to_common());
        halt().await
    }

    // Mode is chosen from VBUS only: USB -> Desk, otherwise Watch.
    let vbus = power.is_vbus_present().unwrap_or(false);
    let mode = if vbus { Mode::Desk } else { Mode::Watch };
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
    let report = persist::init(peripherals.FLASH, mode, battery_mv);
    if mode == Mode::Desk {
        println!(
            "[kv] {} boot {} error {:?}",
            report.label(),
            report.boot(),
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
    if let Some(display) = display.as_mut() {
        if let Err(error) = display.init() {
            println!("[lcd] init failed: {:?}", error);
        } else {
            #[cfg(feature = "trace")]
            {
                if let Err(error) = display.draw_viewer(
                    report.label(),
                    report.boot(),
                    storage::SCHEMA_VERSION,
                    report.error_code(),
                ) {
                    println!("[lcd] viewer failed: {:?}", error);
                }
            }
            #[cfg(not(feature = "trace"))]
            refresh(&mut power, &mut clock, Some(display));
        }
    }

    if let Some(reason) = wake {
        if let Err(error) = power.request_awake(reason, AWAKE_BUDGET_MS, 0) {
            println!("[pm] awake failed: {}", error.to_common());
        }
    }

    if mode == Mode::Desk {
        desk_keepalive(&mut power, &mut clock, display.as_mut()).await;
        if let Err(error) = power.set_mode(Mode::Watch) {
            println!("[pm] set_mode failed: {}", error.to_common());
            halt().await
        }
    }

    // Watch mode: a brief awake window, then deep sleep.
    Timer::after(AWAKE_WINDOW).await;

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
    let mut wake_pins: [&mut dyn RtcPin; 1] = [&mut pwr_button];
    let button = Ext1WakeupSource::new(&mut wake_pins, WakeupLevel::High);
    rtc.sleep_deep(&[&timer, &button]);
}
