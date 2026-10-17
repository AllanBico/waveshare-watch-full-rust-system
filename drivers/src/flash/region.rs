//! A labelled partition viewed as a NOR flash region.

use embedded_storage::nor_flash::{ErrorType, MultiwriteNorFlash, NorFlash, ReadNorFlash};
use embedded_storage::ReadStorage;

use super::{FlashDevice, FlashError};

/// Flash word size used by the ESP32-S3.
pub const WORD_SIZE: u32 = 4;
/// Flash sector size used by the ESP32-S3.
pub const SECTOR_SIZE: u32 = 4096;

/// A partition offset and length on top of [`FlashDevice`].
pub struct PartitionFlash<'a> {
    device: &'a FlashDevice,
    offset: u32,
    len: u32,
}

impl<'a> PartitionFlash<'a> {
    pub(crate) const fn new(device: &'a FlashDevice, offset: u32, len: u32) -> Self {
        Self {
            device,
            offset,
            len,
        }
    }

    /// Absolute flash offset of the partition.
    pub const fn offset(&self) -> u32 {
        self.offset
    }

    /// Partition length in bytes.
    pub const fn len(&self) -> u32 {
        self.len
    }

    /// Whether the partition has zero length.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn covers(&self, offset: u32, length: usize) -> bool {
        offset <= self.len && length <= (self.len - offset) as usize
    }

    fn check_alignment(offset: u32, length: usize, align: u32) -> Result<(), FlashError> {
        if !offset.is_multiple_of(align) || !(length as u32).is_multiple_of(align) {
            return Err(FlashError::NotAligned);
        }
        Ok(())
    }
}

impl ErrorType for PartitionFlash<'_> {
    type Error = FlashError;
}

impl ReadStorage for PartitionFlash<'_> {
    type Error = FlashError;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if !self.covers(offset, bytes.len()) {
            return Err(FlashError::OutOfBounds);
        }
        self.device.read_bytes(self.offset + offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.len as usize
    }
}

impl ReadNorFlash for PartitionFlash<'_> {
    const READ_SIZE: usize = WORD_SIZE as usize;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        Self::check_alignment(offset, bytes.len(), WORD_SIZE)?;
        if !self.covers(offset, bytes.len()) {
            return Err(FlashError::OutOfBounds);
        }
        self.device.read_aligned(self.offset + offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.len as usize
    }
}

impl NorFlash for PartitionFlash<'_> {
    const WRITE_SIZE: usize = WORD_SIZE as usize;
    const ERASE_SIZE: usize = SECTOR_SIZE as usize;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        Self::check_alignment(offset, bytes.len(), WORD_SIZE)?;
        if !self.covers(offset, bytes.len()) {
            return Err(FlashError::OutOfBounds);
        }
        self.device.write_aligned(self.offset + offset, bytes)
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if from > to || to > self.len {
            return Err(FlashError::OutOfBounds);
        }
        Self::check_alignment(from, (to - from) as usize, SECTOR_SIZE)?;
        self.device.erase(self.offset + from, self.offset + to)
    }
}

impl MultiwriteNorFlash for PartitionFlash<'_> {}

impl core::fmt::Debug for PartitionFlash<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PartitionFlash")
            .field("offset", &self.offset)
            .field("len", &self.len)
            .finish()
    }
}
