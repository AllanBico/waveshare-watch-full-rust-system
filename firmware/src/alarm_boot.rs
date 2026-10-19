//! Alarm-store bring-up and the trace alarm fixture.

use static_cell::StaticCell;

use alarms::AlarmStore;
use common::{Mode, Timestamp};
use drivers::flash::{FlashDevice, PartitionFlash};
use storage::{Provisioning, TicKvStorage, REGION_SIZE};

#[cfg(feature = "trace")]
use crate::log_boot::STORAGE_WRITE_MIN_MV;

const ALARM_LABEL: &str = "alarms";
#[cfg(feature = "trace")]
const TRACE_ALARM_DELAY_SECS: u64 = 60;

static ALARM_BUFFER: StaticCell<[u8; REGION_SIZE]> = StaticCell::new();

/// Concrete alarm backend over the `alarms` partition.
pub type Backend = TicKvStorage<'static, PartitionFlash<'static>>;

/// Opens the alarm store, recording failures instead of stopping boot.
pub fn open(device: &'static FlashDevice, mode: Mode) -> AlarmStore<Backend> {
    let region = match device.partition(ALARM_LABEL) {
        Ok(region) => region,
        Err(error) => {
            crate::retained::record_error(error.to_common().code());
            return AlarmStore::unavailable();
        }
    };
    let buffer = ALARM_BUFFER.init([0u8; REGION_SIZE]);
    let provisioning = if mode == Mode::Desk {
        Provisioning::Allowed
    } else {
        Provisioning::Forbidden
    };
    match TicKvStorage::open(region, buffer, provisioning) {
        Ok((store, _)) => {
            let (alarms, load) = AlarmStore::load(store);
            if load.failed {
                crate::retained::record_error(common::Error::StorageCorrupt.code());
            }
            alarms
        }
        Err(error) => {
            crate::retained::record_error(error.to_common().code());
            AlarmStore::unavailable()
        }
    }
}

/// Seeds one alarm a minute ahead when a trace build has no alarm stored.
pub fn seed_trace(
    alarms: AlarmStore<Backend>,
    now: Timestamp,
    battery_mv: u16,
    mode: Mode,
) -> AlarmStore<Backend> {
    #[cfg(feature = "trace")]
    let mut alarms = alarms;
    #[cfg(feature = "trace")]
    {
        if mode == Mode::Desk && battery_mv >= STORAGE_WRITE_MIN_MV && alarms.set().is_empty() {
            let at = Timestamp::from_secs(now.as_secs().saturating_add(TRACE_ALARM_DELAY_SECS));
            if alarms.add(at) {
                let _ = alarms.flush();
            }
        }
    }
    #[cfg(not(feature = "trace"))]
    {
        let _ = (now, battery_mv, mode);
    }
    alarms
}
