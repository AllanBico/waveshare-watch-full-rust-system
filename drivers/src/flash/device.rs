//! The single flash device and its labelled partitions.

use core::cell::RefCell;

use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use embedded_storage::ReadStorage;
use esp_bootloader_esp_idf::partitions::{read_partition_table, PARTITION_TABLE_MAX_LEN};
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;

use super::{FlashError, PartitionFlash};

/// Maximum number of partitions the device keeps. The table format allows
/// more, but this board never needs them.
pub const MAX_PARTITIONS: usize = 16;

/// Partition labels are at most 16 bytes in the partition table format.
const LABEL_LEN: usize = 16;

#[derive(Clone, Copy)]
struct PartitionInfo {
    label: [u8; LABEL_LEN],
    label_len: u8,
    offset: u32,
    len: u32,
}

/// Owns the flash handle and the partition layout.
///
/// Only one device may exist: `FlashStorage::new` panics on a second call.
pub struct FlashDevice {
    flash: RefCell<FlashStorage<'static>>,
    entries: [Option<PartitionInfo>; MAX_PARTITIONS],
}

impl FlashDevice {
    /// Reads the partition table and claims the flash handle.
    pub fn new(flash: FLASH<'static>) -> Result<Self, FlashError> {
        let flash = RefCell::new(FlashStorage::new(flash));
        let mut buffer = [0u8; PARTITION_TABLE_MAX_LEN];
        let table = read_partition_table(&mut *flash.borrow_mut(), &mut buffer)
            .map_err(|_| FlashError::PartitionTable)?;

        let mut entries = [None; MAX_PARTITIONS];
        for (count, entry) in table.iter().enumerate() {
            if count == MAX_PARTITIONS {
                return Err(FlashError::TooManyPartitions);
            }
            let bytes = entry.label_as_str().as_bytes();
            let label_len = u8::try_from(bytes.len()).map_err(|_| FlashError::PartitionTable)?;
            if bytes.len() > LABEL_LEN {
                return Err(FlashError::PartitionTable);
            }
            let mut label = [0u8; LABEL_LEN];
            label[..bytes.len()].copy_from_slice(bytes);
            entries[count] = Some(PartitionInfo {
                label,
                label_len,
                offset: entry.offset(),
                len: entry.len(),
            });
        }

        Ok(Self { flash, entries })
    }

    /// Looks up a partition by its label.
    pub fn partition(&self, label: &str) -> Result<PartitionFlash<'_>, FlashError> {
        let wanted = label.as_bytes();
        for info in self.entries.iter().flatten() {
            if &info.label[..info.label_len as usize] == wanted {
                return Ok(PartitionFlash::new(self, info.offset, info.len));
            }
        }
        Err(FlashError::PartitionNotFound)
    }

    /// Number of partitions read from the table.
    pub fn partition_count(&self) -> usize {
        self.entries.iter().flatten().count()
    }

    pub(crate) fn read_bytes(&self, offset: u32, bytes: &mut [u8]) -> Result<(), FlashError> {
        ReadStorage::read(&mut *self.flash.borrow_mut(), offset, bytes)
            .map_err(|_| FlashError::Storage)
    }

    pub(crate) fn read_aligned(&self, offset: u32, bytes: &mut [u8]) -> Result<(), FlashError> {
        ReadNorFlash::read(&mut *self.flash.borrow_mut(), offset, bytes)
            .map_err(|_| FlashError::Storage)
    }

    pub(crate) fn write_aligned(&self, offset: u32, bytes: &[u8]) -> Result<(), FlashError> {
        NorFlash::write(&mut *self.flash.borrow_mut(), offset, bytes)
            .map_err(|_| FlashError::Storage)
    }

    pub(crate) fn erase(&self, from: u32, to: u32) -> Result<(), FlashError> {
        NorFlash::erase(&mut *self.flash.borrow_mut(), from, to).map_err(|_| FlashError::Storage)
    }
}
