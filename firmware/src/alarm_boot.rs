//! Alarm-store bring-up.

use static_cell::StaticCell;

use alarms::AlarmStore;
use common::Mode;
use drivers::flash::{FlashDevice, PartitionFlash};
use storage::{Provisioning, TicKvStorage, REGION_SIZE};

const ALARM_LABEL: &str = "alarms";

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
