//! TicKV flash controller over a `NorFlash` region.
//!
//! TicKV writes unaligned headers, values, and checksums. The adapter performs
//! word-sized read-modify-write so neighbouring bytes survive. It never erases
//! on write; TicKV asks for region erases explicitly.

use core::cell::RefCell;

use embedded_storage::nor_flash::NorFlash;
use embedded_storage::ReadStorage;
use tickv::flash_controller::FlashController;
use tickv::ErrorCode;

/// TicKV region size, matching the ESP32-S3 flash sector.
pub const REGION_SIZE: usize = 4096;
const REGION_SIZE_U32: u32 = REGION_SIZE as u32;
const WORD_SIZE: usize = 4;
const WORD_SIZE_U32: u32 = WORD_SIZE as u32;
const BLANK_CHUNK: usize = 256;

#[repr(align(4))]
struct Word([u8; WORD_SIZE]);

/// Adapts a NOR flash region to TicKV's block interface.
pub struct Controller<F> {
    flash: RefCell<F>,
}

impl<F: NorFlash + ReadStorage> Controller<F> {
    pub(crate) const fn new(flash: F) -> Self {
        Self {
            flash: RefCell::new(flash),
        }
    }

    fn region_address(region_number: usize) -> Result<u32, ErrorCode> {
        u32::try_from(region_number)
            .ok()
            .and_then(|region| region.checked_mul(REGION_SIZE_U32))
            .ok_or(ErrorCode::ReadFail)
    }

    fn read_word(&self, offset: u32) -> Result<Word, ErrorCode> {
        let mut word = Word([0u8; WORD_SIZE]);
        ReadStorage::read(&mut *self.flash.borrow_mut(), offset, &mut word.0)
            .map_err(|_| ErrorCode::ReadFail)?;
        Ok(word)
    }

    fn write_word(&self, offset: u32, word: Word) -> Result<(), ErrorCode> {
        NorFlash::write(&mut *self.flash.borrow_mut(), offset, &word.0)
            .map_err(|_| ErrorCode::WriteFail)
    }

    fn write_bytes(&self, address: u32, bytes: &[u8]) -> Result<(), ErrorCode> {
        let mut written = 0usize;
        while written < bytes.len() {
            let current = u32::try_from(written)
                .ok()
                .and_then(|offset| address.checked_add(offset))
                .ok_or(ErrorCode::WriteFail)?;
            let word_offset = current & !(WORD_SIZE_U32 - 1);
            let byte_offset = (current % WORD_SIZE_U32) as usize;
            let mut word = self.read_word(word_offset)?;
            let room = WORD_SIZE - byte_offset;
            let count = room.min(bytes.len() - written);
            word.0[byte_offset..byte_offset + count]
                .copy_from_slice(&bytes[written..written + count]);
            self.write_word(word_offset, word)?;
            written += count;
        }
        Ok(())
    }

    /// Whether the whole region is erased.
    ///
    /// Used only on the missing-super-key path so a corrupted store is never
    /// silently reformatted.
    pub(crate) fn is_blank(&self) -> Result<bool, ErrorCode> {
        let mut flash = self.flash.borrow_mut();
        let size = ReadStorage::capacity(&*flash);
        let mut buffer = [0u8; BLANK_CHUNK];
        let mut offset = 0u32;

        while (offset as usize) < size {
            let remaining = size - offset as usize;
            let length = BLANK_CHUNK.min(remaining);
            ReadStorage::read(&mut *flash, offset, &mut buffer[..length])
                .map_err(|_| ErrorCode::ReadFail)?;
            if buffer[..length].iter().any(|byte| *byte != 0xFF) {
                return Ok(false);
            }
            offset += length as u32;
        }
        Ok(true)
    }
}

impl<F: NorFlash + ReadStorage> FlashController<REGION_SIZE> for Controller<F> {
    fn read_region(
        &self,
        region_number: usize,
        buffer: &mut [u8; REGION_SIZE],
    ) -> Result<(), ErrorCode> {
        let offset = Self::region_address(region_number)?;
        ReadStorage::read(&mut *self.flash.borrow_mut(), offset, buffer)
            .map_err(|_| ErrorCode::ReadFail)
    }

    fn write(&self, address: usize, buffer: &[u8]) -> Result<(), ErrorCode> {
        let address = u32::try_from(address).map_err(|_| ErrorCode::WriteFail)?;
        self.write_bytes(address, buffer)
    }

    fn erase_region(&self, region_number: usize) -> Result<(), ErrorCode> {
        let start = Self::region_address(region_number)?;
        let end = start
            .checked_add(REGION_SIZE_U32)
            .ok_or(ErrorCode::EraseFail)?;
        NorFlash::erase(&mut *self.flash.borrow_mut(), start, end).map_err(|_| ErrorCode::EraseFail)
    }
}
