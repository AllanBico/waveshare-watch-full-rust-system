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

use common::{DateTime, MotionSample, Timestamp, TouchReport, WakeReason};
use drivers::board;
use drivers::pcf85063a::Pcf85063a;
use embedded_hal_bus::i2c::RefCellDevice;
use hal::{Imu, Power, Touch};
use power::PowerManager;

use crate::display::{self, Display as Panel};

/// CPU clock for bring-up. Mode-based clock switching comes later.
pub const BRINGUP_CPU_CLOCK_MHZ: u32 = 160;

/// Input poll period while the panel is awake.
pub const INPUT_POLL_PERIOD: Duration = Duration::from_millis(50);

/// Input polls between desk-mode VBUS checks.
const INPUT_POLLS_PER_SECOND: u32 = 20;

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

/// Input devices plus the latest viewer snapshot.
pub struct InputState<'a, T: Touch, I: Imu> {
    /// Touch controller.
    pub touch: &'a mut T,
    /// IMU.
    pub imu: &'a mut I,
    /// IMU chip identity, when it could be read.
    pub imu_id: Option<u8>,
    /// IMU revision, when it could be read.
    pub revision: Option<u8>,
    report: TouchReport,
    motion: MotionSample,
    touch_error: Option<u16>,
    imu_error: Option<u16>,
    log: bool,
}

impl<'a, T: Touch, I: Imu> InputState<'a, T, I> {
    /// Creates an input state from the devices and their initial errors.
    pub fn new(
        touch: &'a mut T,
        imu: &'a mut I,
        touch_error: Option<u16>,
        imu_error: Option<u16>,
        log: bool,
    ) -> Self {
        Self {
            touch,
            imu,
            imu_id: None,
            revision: None,
            report: TouchReport::empty(),
            motion: MotionSample::ZERO,
            touch_error,
            imu_error,
            log,
        }
    }

    fn error(&self) -> Option<u16> {
        self.touch_error.or(self.imu_error)
    }
}

/// Stays awake in desk mode until USB is removed, updating the input viewer.
pub async fn desk_keepalive<P: Power, T: Touch, I: Imu>(
    power: &mut PowerManager<P>,
    display: &mut Option<Panel>,
    input: &mut InputState<'_, T, I>,
) {
    let start = Instant::now();
    let mut ticks = 0u32;
    loop {
        Timer::after(INPUT_POLL_PERIOD).await;
        poll_input(display, input);

        ticks += 1;
        if !ticks.is_multiple_of(INPUT_POLLS_PER_SECOND) {
            continue;
        }
        if !power.is_vbus_present().unwrap_or(true) {
            println!("[pm] USB removed");
            return;
        }
        println!("[hb] uptime {} ms", (Instant::now() - start).as_millis());
    }
}

/// Keeps the panel awake for `window` while sampling touch and motion.
pub async fn watch_awake<T: Touch, I: Imu>(
    display: &mut Option<Panel>,
    input: &mut InputState<'_, T, I>,
    window: Duration,
) {
    let deadline = Instant::now() + window;
    while Instant::now() < deadline {
        Timer::after(INPUT_POLL_PERIOD).await;
        poll_input(display, input);
    }
}

fn poll_input<T: Touch, I: Imu>(display: &mut Option<Panel>, input: &mut InputState<'_, T, I>) {
    let previous_error = input.error();
    let mut touch_changed = false;
    let mut motion_changed = false;

    match input.touch.read() {
        Ok(report) => {
            input.touch_error = None;
            if report != input.report {
                input.report = report;
                touch_changed = true;
            }
        }
        Err(_) => {
            input.touch_error = Some(common::Error::HardwareFailure.code());
            if !input.report.is_empty() {
                input.report = TouchReport::empty();
                touch_changed = true;
            }
        }
    }

    match input.imu.read_motion() {
        Ok(motion) => {
            input.imu_error = None;
            if motion != input.motion {
                input.motion = motion;
                motion_changed = true;
            }
        }
        Err(_) => input.imu_error = Some(common::Error::HardwareFailure.code()),
    }

    let error_changed = input.error() != previous_error;
    if touch_changed || motion_changed || error_changed {
        redraw_input(display, input);
    }
    if input.log && (touch_changed || error_changed) {
        log_touch(input);
    }
}

fn redraw_input<T: Touch, I: Imu>(display: &mut Option<Panel>, input: &InputState<'_, T, I>) {
    if let Some(panel) = display.as_mut() {
        let viewer = display::Viewer {
            touch: &input.report,
            motion: &input.motion,
            imu_id: input.imu_id,
            revision: input.revision,
            error: input.error(),
        };
        let _ = panel.draw_input_viewer(&viewer);
    }
}

fn log_touch<T: Touch, I: Imu>(input: &InputState<'_, T, I>) {
    if let Some(code) = input.error() {
        println!("[tp] error {}", code);
    } else if input.report.is_empty() {
        println!("[tp] none");
    } else {
        for point in input.report.points() {
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

    println!("=== wv2 firmware (Phase 4.2 input bring-up) ===");
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
