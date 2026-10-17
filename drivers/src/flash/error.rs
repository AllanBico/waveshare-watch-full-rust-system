//! Flash driver errors.

use core::fmt;

use common::Error;
use embedded_storage::nor_flash::{NorFlashError, NorFlashErrorKind};

/// Error returned by the flash partition driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum FlashError {
    /// The partition table could not be read or parsed.
    PartitionTable,
    /// No partition carries the requested label.
    PartitionNotFound,
    /// The partition table has more entries than the device can hold.
    TooManyPartitions,
    /// A region access crossed the partition boundary.
    OutOfBounds,
    /// A flash operation was not aligned to its required granularity.
    NotAligned,
    /// The underlying flash reported a failure.
    Storage,
}

impl FlashError {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(self) -> Error {
        match self {
            FlashError::PartitionTable => Error::StorageCorrupt,
            FlashError::PartitionNotFound
            | FlashError::TooManyPartitions
            | FlashError::OutOfBounds
            | FlashError::NotAligned => Error::InvalidInput,
            FlashError::Storage => Error::HardwareFailure,
        }
    }
}

impl NorFlashError for FlashError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            FlashError::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            FlashError::NotAligned => NorFlashErrorKind::NotAligned,
            _ => NorFlashErrorKind::Other,
        }
    }
}

impl fmt::Display for FlashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            FlashError::PartitionTable => "partition table invalid",
            FlashError::PartitionNotFound => "partition not found",
            FlashError::TooManyPartitions => "partition table too large",
            FlashError::OutOfBounds => "partition access out of bounds",
            FlashError::NotAligned => "partition access not aligned",
            FlashError::Storage => "flash storage error",
        })
    }
}
