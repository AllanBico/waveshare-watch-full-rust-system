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

use common::{DateTime, Timestamp, TouchReport, WakeReason};
use drivers::board;
use drivers::pcf85063a::Pcf85063a;
use embedded_hal_bus::i2c::RefCellDevice;
use hal::{Power, Touch};
use power::PowerManager;

use crate::display::{self, Display as Panel};

/// CPU clock for bring-up. Mode-based clock switching comes later.
pub const BRINGUP_CPU_CLOCK_MHZ: u32 = 160;

/// Touch poll period while the panel is awake.
pub const TOUCH_POLL_PERIOD: Duration = Duration::from_millis(50);

/// Touch polls between desk-mode VBUS checks.
const TOUCH_POLLS_PER_SECOND: u32 = 20;

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

/// State shared by the touch viewer in both awake modes.
struct ViewerState {
    report: TouchReport,
    chip_id: Option<u8>,
    error: Option<u16>,
    log: bool,
}

impl ViewerState {
    fn new(chip_id: Option<u8>, error: Option<u16>, log: bool) -> Self {
        Self {
            report: TouchReport::empty(),
            chip_id,
            error,
            log,
        }
    }
}

/// Stays awake in desk mode until USB is removed, updating the touch viewer.
pub async fn desk_keepalive<P: Power, T: Touch>(
    power: &mut PowerManager<P>,
    display: &mut Option<Panel>,
    touch: &mut T,
    chip_id: Option<u8>,
    error: Option<u16>,
) {
    let start = Instant::now();
    let mut state = ViewerState::new(chip_id, error, true);
    let mut ticks = 0u32;
    loop {
        Timer::after(TOUCH_POLL_PERIOD).await;
        poll_touch(display, touch, &mut state);

        ticks += 1;
        if !ticks.is_multiple_of(TOUCH_POLLS_PER_SECOND) {
            continue;
        }
        if !power.is_vbus_present().unwrap_or(true) {
            println!("[pm] USB removed");
            return;
        }
        println!("[hb] uptime {} ms", (Instant::now() - start).as_millis());
    }
}

/// Keeps the panel awake for `window` while sampling touch.
pub async fn watch_awake<T: Touch>(
    display: &mut Option<Panel>,
    touch: &mut T,
    chip_id: Option<u8>,
    error: Option<u16>,
    window: Duration,
) {
    let deadline = Instant::now() + window;
    let mut state = ViewerState::new(chip_id, error, false);
    while Instant::now() < deadline {
        Timer::after(TOUCH_POLL_PERIOD).await;
        poll_touch(display, touch, &mut state);
    }
}

fn poll_touch<T: Touch>(display: &mut Option<Panel>, touch: &mut T, state: &mut ViewerState) {
    let previous_error = state.error;
    let next = match touch.read() {
        Ok(report) => {
            state.error = None;
            report
        }
        Err(_) => {
            state.error = Some(common::Error::HardwareFailure.code());
            TouchReport::empty()
        }
    };

    if next == state.report && state.error == previous_error {
        return;
    }
    state.report = next;
    redraw_touch(display, state);
    if state.log {
        log_touch(state);
    }
}

fn redraw_touch(display: &mut Option<Panel>, state: &ViewerState) {
    if let Some(panel) = display.as_mut() {
        let viewer = display::Viewer {
            report: &state.report,
            chip_id: state.chip_id,
            error: state.error,
        };
        let _ = panel.draw_touch_viewer(&viewer);
    }
}

fn log_touch(state: &ViewerState) {
    if let Some(code) = state.error {
        println!("[tp] error {}", code);
    } else if state.report.is_empty() {
        println!("[tp] none");
    } else {
        for point in state.report.points() {
            println!("[tp] id {} x {} y {}", point.id, point.x, point.y);
        }
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

    println!("=== wv2 firmware (Phase 4.1 touch bring-up) ===");
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
