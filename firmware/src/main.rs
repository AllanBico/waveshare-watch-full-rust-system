//! Firmware entry point.
//!
//! Phase 1.5: the AXP2101 and the PCF85063A share one I2C bus. Mode comes from
//! VBUS (USB -> Desk, else Watch). Watch mode deep-sleeps, waking on the RTC
//! timer or the power button. The external RTC keeps wall-clock time.

#![no_std]
#![no_main]

extern crate alloc;

use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::efuse::Efuse;
use esp_hal::gpio::RtcPin;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel};
use esp_hal::rtc_cntl::{wakeup_cause, Rtc};
use esp_hal::system::SleepSource;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

use common::{DateTime, Mode, WakeReason};
use drivers::axp2101::Axp2101;
use drivers::board;
use drivers::pcf85063a::Pcf85063a;

mod display;
use embedded_hal_bus::i2c::RefCellDevice;
use hal::Power;
use power::PowerManager;

use esp_alloc as _;
use esp_backtrace as _;

esp_bootloader_esp_idf::esp_app_desc!();

/// CPU clock for bring-up. Mode-based clock switching comes later.
const BRINGUP_CPU_CLOCK_MHZ: u32 = 160;

/// Awake time budget granted on each watch-mode wake.
const AWAKE_BUDGET_MS: u32 = 5_000;

/// How long to stay awake before sleeping, for the 1.5 bring-up.
const AWAKE_WINDOW: Duration = Duration::from_secs(2);

/// Desk-mode VBUS poll and heartbeat period.
const POLL_PERIOD: Duration = Duration::from_secs(1);

/// SRAM heap size reserved for the runtime.
const SRAM_HEAP_BYTES: usize = 64 * 1024;

/// I2C bus device handle shared by all on-board peripherals.
type Bus = RefCellDevice<'static, I2c<'static, esp_hal::Blocking>>;

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
    let datetime = report_time(&mut clock);

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

    // Phase 3.4: own the panel across the awake window so it can sleep.
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
        let battery_mv = match power.pmic_mut().read_battery_millivolts() {
            Ok(millivolts) => millivolts.value(),
            Err(error) => {
                println!("[pm] battery read failed: {:?}", error);
                0
            }
        };
        println!("[pm] battery {} mV", battery_mv);
        match display.init_and_draw(datetime.hour, datetime.minute, battery_mv) {
            Ok(stat) => println!("[lcd] lit {}/{} pixels", stat.lit_pixels, stat.total_pixels),
            Err(error) => println!("[lcd] draw failed: {:?}", error),
        }
    }

    if let Some(reason) = wake {
        if let Err(error) = power.request_awake(reason, AWAKE_BUDGET_MS, 0) {
            println!("[pm] awake failed: {}", error.to_common());
        }
    }

    if mode == Mode::Desk {
        desk_keepalive(&mut power).await;
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

/// Reads the RTC time; seeds a default if the oscillator had stopped, and
/// returns the effective date/time.
fn report_time(clock: &mut Pcf85063a<Bus>) -> DateTime {
    let default = DateTime::new(2000, 1, 1, 0, 0, 0);
    if let Err(error) = clock.init() {
        println!("[rtc] init failed: {}", error);
        return default;
    }
    match clock.read_datetime() {
        Ok(dt) => {
            print_datetime("time", &dt);
            dt
        }
        Err(_) => {
            if clock.set_datetime(&default).is_ok() {
                print_datetime("time set", &default);
            } else {
                println!("[rtc] set failed");
            }
            default
        }
    }
}

/// Prints a date/time.
fn print_datetime(label: &str, dt: &DateTime) {
    println!(
        "[rtc] {} {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        label, dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
    );
}

/// Opens and leaks the shared I2C bus so device handles can be `'static`.
async fn open_bus(
    i2c: esp_hal::peripherals::I2C0<'static>,
    sda: esp_hal::peripherals::GPIO15<'static>,
    scl: esp_hal::peripherals::GPIO14<'static>,
) -> &'static RefCell<I2c<'static, esp_hal::Blocking>> {
    let config = I2cConfig::default().with_frequency(Rate::from_khz(board::I2C_FREQ_HZ / 1000));
    match I2c::new(i2c, config) {
        Ok(bus) => alloc::boxed::Box::leak(alloc::boxed::Box::new(RefCell::new(
            bus.with_sda(sda).with_scl(scl),
        ))),
        Err(error) => {
            println!("[pm] i2c init failed: {:?}", error);
            halt().await
        }
    }
}

/// Stays awake in desk mode until USB is removed.
async fn desk_keepalive<P: Power>(power: &mut PowerManager<P>) {
    let start = Instant::now();
    loop {
        Timer::after(POLL_PERIOD).await;
        if !power.is_vbus_present().unwrap_or(true) {
            println!("[pm] USB removed");
            return;
        }
        println!("[hb] uptime {} ms", (Instant::now() - start).as_millis());
    }
}

/// Maps the SoC wake cause onto a shared wake reason.
fn wake_reason() -> Option<WakeReason> {
    match wakeup_cause() {
        SleepSource::Timer => Some(WakeReason::Timer),
        SleepSource::Ext0 | SleepSource::Ext1 => Some(WakeReason::Button),
        SleepSource::Undefined => None,
        _ => Some(WakeReason::Unknown),
    }
}

/// Idles forever after a fatal bring-up error.
async fn halt() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

/// Prints the silicon facts read at runtime, plus the PSRAM size.
fn report_boot(psram_bytes: usize) {
    let revision = Efuse::chip_revision();
    let mac = Efuse::mac_address();

    println!("=== wv2 firmware (Phase 1.5 RTC bring-up) ===");
    println!(
        "chip:  {} rev v{}.{}",
        esp_hal::chip!(),
        revision / 100,
        revision % 100
    );
    println!("cpu:   {} MHz", BRINGUP_CPU_CLOCK_MHZ);
    println!(
        "mac:   {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    println!("psram: {} bytes", psram_bytes);
    match esp_hal::system::reset_reason() {
        Some(reason) => println!("reset: {:?}", reason),
        None => println!("reset: unknown"),
    }
}
