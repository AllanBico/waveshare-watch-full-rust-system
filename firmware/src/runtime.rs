//! Runtime helpers: bus setup, boot reporting, the awake refresh loop, and the
//! RTC/battery readout. `main` owns the orchestration; this module owns the
//! pieces it calls.

use core::cell::RefCell;

use embassy_time::{Duration, Timer};
use esp_hal::efuse::Efuse;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::peripherals::GPIO10;
use esp_hal::rtc_cntl::wakeup_cause;
use esp_hal::system::SleepSource;
use esp_hal::time::Rate;
use esp_println::println;
use static_cell::StaticCell;

use common::{DateTime, Mode, Timestamp, WakeReason};
use drivers::board;
use drivers::pcf85063a::Pcf85063a;
use embedded_hal_bus::i2c::RefCellDevice;

/// CPU clock for the shared boot chain. Both modes boot watch-safe.
pub const BOOT_CPU_CLOCK_MHZ: u32 = 80;

/// Number of PWR button samples used for the boot debounce.
pub const BUTTON_SAMPLES: u8 = 4;

/// Spacing between PWR button samples.
pub const BUTTON_SAMPLE_PERIOD: Duration = Duration::from_millis(5);

/// I2C bus device handle shared by all on-board peripherals.
pub type Bus = RefCellDevice<'static, I2c<'static, esp_hal::Blocking>>;

static BUS: StaticCell<RefCell<I2c<'static, esp_hal::Blocking>>> = StaticCell::new();

/// Opens the shared I2C bus in static storage so device handles can be
/// `'static` without a heap allocation on the wake path.
pub async fn open_bus(
    i2c: esp_hal::peripherals::I2C0<'static>,
    sda: esp_hal::peripherals::GPIO15<'static>,
    scl: esp_hal::peripherals::GPIO14<'static>,
) -> &'static RefCell<I2c<'static, esp_hal::Blocking>> {
    let config = I2cConfig::default().with_frequency(Rate::from_khz(board::I2C_FREQ_HZ / 1000));
    match I2c::new(i2c, config) {
        Ok(bus) => BUS.init(RefCell::new(bus.with_sda(sda).with_scl(scl))),
        Err(error) => {
            println!("[pm] i2c init failed: {:?}", error);
            halt().await
        }
    }
}

/// Reads the RTC time; seeds a default if the oscillator had stopped.
///
/// Serial output is desk-only; watch keeps the retained counters instead.
pub fn report_time(clock: &mut Pcf85063a<Bus>, log: bool) {
    let default = DateTime::new(2000, 1, 1, 0, 0, 0);
    if let Err(error) = clock.init() {
        if log {
            println!("[rtc] init failed: {}", error);
        }
        return;
    }
    match clock.read_datetime() {
        Ok(dt) => {
            if log {
                print_datetime("time", &dt);
            }
        }
        Err(_) => {
            if clock.set_datetime(&default).is_ok() {
                if log {
                    print_datetime("time set", &default);
                }
            } else if log {
                println!("[rtc] set failed");
            }
        }
    }
}

/// Reads the RTC as a Unix timestamp, falling back to zero if unavailable.
pub fn read_timestamp(clock: &mut Pcf85063a<Bus>) -> Timestamp {
    clock
        .read_datetime()
        .ok()
        .and_then(|datetime| datetime.to_timestamp())
        .unwrap_or(Timestamp::from_secs(0))
}

/// Prints a date/time.
fn print_datetime(label: &str, dt: &DateTime) {
    println!(
        "[rtc] {} {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        label, dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second
    );
}

/// Maps the SoC wake cause onto a coarse shared wake reason.
///
/// Ext1 is shared by the PWR button and the IMU INT1. `main` refines it to
/// [`WakeReason::Motion`] when the IMU reports a Wake-on-Motion event.
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

/// Prints the silicon facts read at runtime, plus the selected mode and PSRAM.
pub fn report_boot(mode: Mode, psram_bytes: usize, log: bool) {
    if !log {
        return;
    }
    let revision = Efuse::chip_revision();
    let mac = Efuse::mac_address();

    println!("=== wv2 firmware (Phase 7.2 boot chain) ===");
    println!(
        "chip:  {} rev v{}.{}",
        esp_hal::chip!(),
        revision / 100,
        revision % 100
    );
    println!("cpu:   {} MHz", BOOT_CPU_CLOCK_MHZ);
    println!("mode:  {:?}", mode);
    println!(
        "mac:   {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    if psram_bytes == 0 {
        println!("psram: disabled");
    } else {
        println!("psram: {} bytes", psram_bytes);
    }
    match esp_hal::system::reset_reason() {
        Some(reason) => println!("reset: {:?}", reason),
        None => println!("reset: unknown"),
    }
}

/// Samples the PWR button to reject a bounce on the wake edge.
pub async fn debounce_button(pin: &Input<'_>) -> bool {
    let mut high = 0u8;
    for _ in 0..BUTTON_SAMPLES {
        if pin.is_high() {
            high += 1;
        }
        Timer::after(BUTTON_SAMPLE_PERIOD).await;
    }
    high * 2 > BUTTON_SAMPLES
}

/// Measures the PWR button only on a button wake, returning the pin for the
/// sleep configuration.
pub async fn debounce_power_button(
    mut pin: GPIO10<'static>,
    raw_wake: Option<WakeReason>,
) -> (GPIO10<'static>, bool) {
    if !matches!(raw_wake, Some(WakeReason::Button)) {
        return (pin, false);
    }
    let input = Input::new(pin.reborrow(), InputConfig::default().with_pull(Pull::Up));
    let pressed = debounce_button(&input).await;
    (pin, pressed)
}
