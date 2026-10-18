//! Storage boot probe and trace selftest.
//!
//! Production boots probe the key-value store, load runtime settings, and run
//! the retained log flush. The trace build also increments a persisted boot
//! counter so the display viewer can prove storage persistence without serial.

use esp_hal::peripherals::FLASH;
use esp_println::println;
use static_cell::StaticCell;

use common::{Mode, Timestamp, WakeReason};
use drivers::flash::{FlashDevice, PartitionFlash};
use settings::SettingsStore;
use storage::{OpenOutcome, Provisioning, TicKvStorage, REGION_SIZE};

use crate::log_boot;
#[cfg(feature = "trace")]
use crate::log_boot::STORAGE_WRITE_MIN_MV;

/// Partition label reserved for TicKV key-value data.
const KV_LABEL: &str = "kv";
/// Key holding the trace boot counter.
#[cfg(feature = "trace")]
const BOOT_KEY: &[u8] = b"selftest/boot";

static READ_BUFFER: StaticCell<[u8; REGION_SIZE]> = StaticCell::new();
static DEVICE: StaticCell<FlashDevice> = StaticCell::new();

/// Concrete settings backend over the `kv` partition.
pub type SettingsBackend = TicKvStorage<'static, PartitionFlash<'static>>;

/// Storage and settings produced by the boot probe.
pub struct Boot {
    /// Boot report for the viewer and serial log.
    pub report: Report,
    /// Loaded settings, or defaults when the store is unavailable.
    pub settings: SettingsStore<SettingsBackend>,
}

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

/// Boot-time storage and logging report.
#[derive(Debug, Clone, Copy)]
pub struct Report {
    state: State,
    boot: u32,
    log: Option<u32>,
    battery_mv: u16,
    buffered: u16,
    dropped: u16,
    error: Option<u16>,
}

impl Report {
    const fn new(state: State, boot: u32, error: Option<u16>) -> Self {
        Self {
            state,
            boot,
            log: None,
            battery_mv: 0,
            buffered: 0,
            dropped: 0,
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

    /// Latest persisted log sequence.
    pub const fn log(self) -> Option<u32> {
        self.log
    }

    /// Battery voltage read this boot.
    pub const fn battery_mv(self) -> u16 {
        self.battery_mv
    }

    /// Records still waiting in retained memory.
    pub const fn buffered(self) -> u16 {
        self.buffered
    }

    /// Records dropped by ring overflow.
    pub const fn dropped(self) -> u16 {
        self.dropped
    }

    /// Stable error code, when the boot failed.
    pub const fn error_code(self) -> Option<u16> {
        self.error
    }
}

/// Opens storage after mode selection, logs the boot sample, and loads settings.
pub fn init(
    flash: FLASH<'static>,
    mode: Mode,
    battery_mv: u16,
    wake: Option<WakeReason>,
    now: Timestamp,
) -> Boot {
    let device = match FlashDevice::new(flash) {
        Ok(device) => DEVICE.init(device),
        Err(error) => {
            return Boot {
                report: failed(mode, error.to_common(), "flash device"),
                settings: SettingsStore::unavailable(),
            };
        }
    };

    let (mut report, opened) = open_kv(device, mode, battery_mv);
    let logs = log_boot::run(device, mode, battery_mv, wake, now);
    report.log = logs.latest;
    report.battery_mv = battery_mv;
    report.buffered = logs.buffered;
    report.dropped = logs.dropped;

    let settings = match opened {
        Some(store) => {
            let (settings, load) = SettingsStore::load(store);
            if mode == Mode::Desk {
                println!(
                    "[set] loaded {} missing {} repaired {}",
                    load.loaded, load.missing, load.repaired
                );
            }
            settings
        }
        None => SettingsStore::unavailable(),
    };

    Boot { report, settings }
}

fn open_kv(
    device: &'static FlashDevice,
    mode: Mode,
    battery_mv: u16,
) -> (Report, Option<SettingsBackend>) {
    let region = match device.partition(KV_LABEL) {
        Ok(region) => region,
        Err(error) => return (failed(mode, error.to_common(), "kv partition"), None),
    };

    let buffer = READ_BUFFER.init([0u8; REGION_SIZE]);
    let provisioning = if mode == Mode::Desk {
        Provisioning::Allowed
    } else {
        Provisioning::Forbidden
    };

    let (mut store, outcome) = match TicKvStorage::open(region, buffer, provisioning) {
        Ok(opened) => opened,
        Err(error) => return (failed(mode, error.to_common(), "open"), None),
    };

    let base = match outcome {
        OpenOutcome::Ready => State::Ready,
        OpenOutcome::Provisioned => State::Provisioned,
    };

    let report = {
        #[cfg(feature = "trace")]
        {
            selftest(&mut store, base, mode, battery_mv)
        }
        #[cfg(not(feature = "trace"))]
        {
            let _ = (&mut store, battery_mv);
            Report::new(base, 0, None)
        }
    };

    (report, Some(store))
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
