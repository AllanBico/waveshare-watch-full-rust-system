//! Runtime helpers: bus setup, boot reporting, the awake refresh loop, and the
//! RTC/battery readout. `main` owns the orchestration; this module owns the
//! pieces it calls.

use core::cell::RefCell;

use embassy_time::{Duration, Instant, Timer};
use esp_hal::efuse::Efuse;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::rtc_cntl::wakeup_cause;
use esp_hal::system::SleepSource;
use esp_hal::time::Rate;
use esp_println::println;

use common::{DateTime, WakeReason};
use drivers::board;
use drivers::pcf85063a::Pcf85063a;
use embedded_hal_bus::i2c::RefCellDevice;
use hal::Power;
use power::PowerManager;

/// CPU clock for bring-up. Mode-based clock switching comes later.
pub const BRINGUP_CPU_CLOCK_MHZ: u32 = 160;

/// Desk-mode VBUS poll and refresh period.
pub const POLL_PERIOD: Duration = Duration::from_secs(1);

/// I2C bus device handle shared by all on-board peripherals.
pub type Bus = RefCellDevice<'static, I2c<'static, esp_hal::Blocking>>;

/// Opens and leaks the shared I2C bus so device handles can be `'static`.
pub async fn open_bus(
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

/// Reads the RTC time; seeds a default if the oscillator had stopped.
pub fn report_time(clock: &mut Pcf85063a<Bus>) {
    let default = DateTime::new(2000, 1, 1, 0, 0, 0);
    if let Err(error) = clock.init() {
        println!("[rtc] init failed: {}", error);
        return;
    }
    match clock.read_datetime() {
        Ok(dt) => print_datetime("time", &dt),
        Err(_) => {
            if clock.set_datetime(&default).is_ok() {
                print_datetime("time set", &default);
            } else {
                println!("[rtc] set failed");
            }
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

/// Stays awake in desk mode until USB is removed, keeping the viewer on screen.
pub async fn desk_keepalive<P: Power>(power: &mut PowerManager<P>) {
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
pub fn wake_reason() -> Option<WakeReason> {
    match wakeup_cause() {
        SleepSource::Timer => Some(WakeReason::Timer),
        SleepSource::Ext0 | SleepSource::Ext1 => Some(WakeReason::Button),
        SleepSource::Undefined => None,
        _ => Some(WakeReason::Unknown),
    }
}

/// Idles forever after a fatal bring-up error.
pub async fn halt() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

/// Prints the silicon facts read at runtime, plus the PSRAM size.
pub fn report_boot(psram_bytes: usize) {
    let revision = Efuse::chip_revision();
    let mac = Efuse::mac_address();

    println!("=== wv2 firmware (Phase 3.4 display bring-up) ===");
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
