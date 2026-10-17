//! Storage boot probe and trace selftest.
//!
//! Production boots only probe. The trace build increments a persisted boot
//! counter and appends a log entry so the display viewer can prove persistence
//! without serial.

use esp_hal::peripherals::FLASH;
use esp_println::println;
use static_cell::StaticCell;

#[cfg(feature = "trace")]
use common::{Event, LogEntry, Timestamp};
use common::{Mode, WakeReason};
use drivers::flash::FlashDevice;
#[cfg(feature = "trace")]
use drivers::flash::PartitionFlash;
#[cfg(feature = "trace")]
use storage::TimeSeries;
use storage::{OpenOutcome, Provisioning, TicKvStorage, REGION_SIZE};

/// Partition label reserved for TicKV key-value data.
const KV_LABEL: &str = "kv";
/// Partition label reserved for the time-series log.
#[cfg(feature = "trace")]
const LOG_LABEL: &str = "logs";
/// Key holding the trace boot counter.
#[cfg(feature = "trace")]
const BOOT_KEY: &[u8] = b"selftest/boot";
/// Below this voltage a flash write is not worth the risk.
#[cfg(feature = "trace")]
const STORAGE_WRITE_MIN_MV: u16 = 3_500;

static READ_BUFFER: StaticCell<[u8; REGION_SIZE]> = StaticCell::new();
#[cfg(feature = "trace")]
static LOG_BUFFER: StaticCell<[u8; REGION_SIZE]> = StaticCell::new();

/// Outcome shown by the viewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Existing store opened.
    Ready,
    /// Blank store provisioned this boot.
    Provisioned,
    /// Battery too low for the trace write.
    #[cfg(feature = "trace")]
    LowBattery,
    /// Store not provisioned and watch mode refused to format it.
    Unavailable,
    /// Store exists but failed its integrity checks.
    Corrupt,
    /// Another storage failure.
    Error,
}

impl State {
    /// Short viewer label.
    pub const fn label(self) -> &'static str {
        match self {
            State::Ready => "KV OK",
            State::Provisioned => "KV NEW",
            #[cfg(feature = "trace")]
            State::LowBattery => "KV LOW BATT",
            State::Unavailable => "KV NONE",
            State::Corrupt => "KV CORRUPT",
            State::Error => "KV ERROR",
        }
    }
}

/// Boot-time storage report.
#[derive(Debug, Clone, Copy)]
pub struct Report {
    state: State,
    boot: u32,
    log: Option<u32>,
    error: Option<u16>,
}

impl Report {
    const fn new(state: State, boot: u32, error: Option<u16>) -> Self {
        Self {
            state,
            boot,
            log: None,
            error,
        }
    }

    /// Viewer label.
    pub const fn label(self) -> &'static str {
        self.state.label()
    }

    /// Persisted trace boot counter.
    pub const fn boot(self) -> u32 {
        self.boot
    }

    /// Latest log sequence, when the time-series selftest ran.
    pub const fn log(self) -> Option<u32> {
        self.log
    }

    /// Stable error code, when the boot failed.
    pub const fn error_code(self) -> Option<u16> {
        self.error
    }
}

/// Opens storage after mode selection and runs the trace selftest.
pub fn init(
    flash: FLASH<'static>,
    mode: Mode,
    battery_mv: u16,
    wake: Option<WakeReason>,
) -> Report {
    let device = match FlashDevice::new(flash) {
        Ok(device) => device,
        Err(error) => return failed(mode, error.to_common(), "flash device"),
    };

    let region = match device.partition(KV_LABEL) {
        Ok(region) => region,
        Err(error) => return failed(mode, error.to_common(), "kv partition"),
    };

    let buffer = READ_BUFFER.init([0u8; REGION_SIZE]);
    let provisioning = if mode == Mode::Desk {
        Provisioning::Allowed
    } else {
        Provisioning::Forbidden
    };

    let (mut store, outcome) = match TicKvStorage::open(region, buffer, provisioning) {
        Ok(opened) => opened,
        Err(error) => return failed(mode, error.to_common(), "open"),
    };

    let base = match outcome {
        OpenOutcome::Ready => State::Ready,
        OpenOutcome::Provisioned => State::Provisioned,
    };

    #[cfg(feature = "trace")]
    {
        let report = selftest(&mut store, base, mode, battery_mv);
        logs_selftest(&device, mode, battery_mv, wake, report)
    }

    #[cfg(not(feature = "trace"))]
    {
        let _ = (&mut store, battery_mv, wake);
        Report::new(base, 0, None)
    }
}

#[cfg(feature = "trace")]
fn selftest(
    store: &mut TicKvStorage<'_, PartitionFlash<'_>>,
    base: State,
    mode: Mode,
    battery_mv: u16,
) -> Report {
    let mut raw = [0u8; 4];
    let boot = match store.get(BOOT_KEY, &mut raw) {
        Ok(4) => u32::from_le_bytes(raw),
        Ok(_) => 0,
        Err(storage::Error::NotFound) => 0,
        Err(error) => return failed(mode, error.to_common(), "selftest read"),
    };

    if battery_mv < STORAGE_WRITE_MIN_MV {
        return Report::new(State::LowBattery, boot, None);
    }

    let next = boot.saturating_add(1);
    match store.set(BOOT_KEY, &next.to_le_bytes()) {
        Ok(()) => Report::new(base, next, None),
        Err(error) => failed(mode, error.to_common(), "selftest write"),
    }
}

#[cfg(feature = "trace")]
fn logs_selftest(
    device: &FlashDevice,
    mode: Mode,
    battery_mv: u16,
    wake: Option<WakeReason>,
    mut report: Report,
) -> Report {
    if battery_mv < STORAGE_WRITE_MIN_MV || report.error_code().is_some() {
        return report;
    }

    let region = match device.partition(LOG_LABEL) {
        Ok(region) => region,
        Err(error) => return log_failed(mode, error.to_common(), "logs partition", report),
    };

    let buffer = LOG_BUFFER.init([0u8; REGION_SIZE]);
    let provisioning = if mode == Mode::Desk {
        Provisioning::Force
    } else {
        Provisioning::Forbidden
    };
    let (mut series, _) = match TimeSeries::open(region, buffer, provisioning) {
        Ok(opened) => opened,
        Err(error) => return log_failed(mode, error.to_common(), "logs open", report),
    };

    let entry = LogEntry::new(
        Timestamp::from_secs(u64::from(report.boot())),
        Event::Wake(wake.unwrap_or(WakeReason::Unknown)),
    );
    let sequence = match series.append_log(entry) {
        Ok(sequence) => sequence,
        Err(error) => return log_failed(mode, error.to_common(), "log append", report),
    };

    let checkpoint = sequence.saturating_sub(1);
    match series.read_logs_since(checkpoint, |_| true) {
        Ok(last) if last == sequence => report.log = u32::try_from(sequence).ok(),
        Ok(_) => {}
        Err(error) => report = log_failed(mode, error.to_common(), "log read", report),
    }
    report
}

#[cfg(feature = "trace")]
fn log_failed(mode: Mode, error: common::Error, what: &str, report: Report) -> Report {
    if mode == Mode::Desk {
        println!("[log] {} failed: {}", what, error);
    }
    report
}

fn failed(mode: Mode, error: common::Error, what: &str) -> Report {
    if mode == Mode::Desk {
        println!("[kv] {} failed: {}", what, error);
    }
    let state = match error {
        common::Error::StorageCorrupt => State::Corrupt,
        common::Error::NotReady => State::Unavailable,
        _ => State::Error,
    };
    Report::new(state, 0, Some(error.code()))
}
