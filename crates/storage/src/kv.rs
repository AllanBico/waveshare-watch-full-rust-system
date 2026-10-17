//! TicKV-backed key-value store.

use embedded_storage::nor_flash::NorFlash;
use embedded_storage::ReadStorage;
use tickv::{ErrorCode, TicKV};

use hal::Storage as StorageTrait;

use super::controller::{Controller, REGION_SIZE};
use super::error::Error;
use super::hash::{hash_key, MAIN_KEY_HASH};

/// Maximum key length accepted at the boundary.
pub const MAX_KEY_LEN: usize = 128;
/// Storage schema version stored under the metadata key.
pub const SCHEMA_VERSION: u16 = 1;

// TicKV asserts `object_length < 0xFFF` when building the header.
const TICKV_MAX_OBJECT: usize = 0x0FFE;
const TICKV_HEADER_LEN: usize = 11;
const TICKV_CHECKSUM_LEN: usize = 4;
/// Largest value TicKV can hold in one object.
pub const MAX_VALUE_LEN: usize = TICKV_MAX_OBJECT - TICKV_HEADER_LEN - TICKV_CHECKSUM_LEN;

const SCHEMA_KEY: &[u8] = b"meta/schema";

/// Whether a missing store may be formatted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provisioning {
    /// Format only a fully erased partition.
    Allowed,
    /// Refuse to format; report not ready instead.
    Forbidden,
    /// Format even a non-blank partition. Destructive; use only for
    /// disposable stores such as logs.
    Force,
}

/// Result of opening a store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenOutcome {
    /// An existing store was opened.
    Ready,
    /// A blank partition was formatted for first use.
    Provisioned,
}

/// A TicKV key-value store over one flash region.
pub struct TicKvStorage<'a, F: NorFlash + ReadStorage> {
    kv: TicKV<'a, Controller<F>, REGION_SIZE>,
}

impl<'a, F: NorFlash + ReadStorage> TicKvStorage<'a, F> {
    /// Opens the store.
    ///
    /// A missing super key on a non-blank partition is treated as corruption,
    /// never as permission to erase, unless the caller explicitly chooses
    /// `Provisioning::Force` for a disposable store.
    pub fn open(
        region: F,
        buffer: &'a mut [u8; REGION_SIZE],
        provisioning: Provisioning,
    ) -> Result<(Self, OpenOutcome), Error> {
        let flash_size = ReadStorage::capacity(&region);
        if flash_size == 0 || !flash_size.is_multiple_of(REGION_SIZE) {
            return Err(Error::InvalidRegion);
        }

        let kv = TicKV::new(Controller::new(region), buffer, flash_size);
        let outcome = match kv.get_key(MAIN_KEY_HASH, &mut []) {
            Ok(_) => OpenOutcome::Ready,
            Err(ErrorCode::KeyNotFound) => {
                match provisioning {
                    Provisioning::Forbidden => return Err(Error::NotReady),
                    Provisioning::Allowed => {
                        if !kv.controller.is_blank().map_err(map_tickv)? {
                            return Err(Error::Corrupt);
                        }
                    }
                    Provisioning::Force => {}
                }
                kv.initialise(MAIN_KEY_HASH).map_err(map_tickv)?;
                OpenOutcome::Provisioned
            }
            Err(code) => return Err(map_tickv(code)),
        };

        let mut store = Self { kv };
        match outcome {
            OpenOutcome::Provisioned => {
                store.set(SCHEMA_KEY, &SCHEMA_VERSION.to_le_bytes())?;
            }
            OpenOutcome::Ready => {
                let mut version = [0u8; 2];
                match store.get(SCHEMA_KEY, &mut version) {
                    Ok(2) if u16::from_le_bytes(version) == SCHEMA_VERSION => {}
                    Ok(2) => return Err(Error::VersionMismatch),
                    Ok(_) => return Err(Error::Corrupt),
                    Err(Error::NotFound) => return Err(Error::NotReady),
                    Err(error) => return Err(error),
                }
            }
        }
        Ok((store, outcome))
    }

    /// Reads a value into `value`, returning its length.
    pub fn get(&mut self, key: &[u8], value: &mut [u8]) -> Result<usize, Error> {
        validate_key(key)?;
        match self.kv.get_key(hash_key(key), value) {
            Ok((_, length)) => Ok(length),
            Err(code) => Err(map_tickv(code)),
        }
    }

    /// Writes or replaces a value.
    pub fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Error> {
        validate_key(key)?;
        if value.len() > MAX_VALUE_LEN {
            return Err(Error::ValueTooLarge);
        }

        let hash = hash_key(key);
        match self.kv.invalidate_key(hash) {
            Ok(_) | Err(ErrorCode::KeyNotFound) => {}
            Err(code) => return Err(map_tickv(code)),
        }

        match self.kv.append_key(hash, value) {
            Ok(_) => Ok(()),
            Err(ErrorCode::FlashFull) => {
                self.kv.garbage_collect().map_err(map_tickv)?;
                self.kv
                    .append_key(hash, value)
                    .map_err(map_tickv)
                    .map(|_| ())
            }
            Err(code) => Err(map_tickv(code)),
        }
    }

    /// Deletes a key, returning whether it existed.
    pub fn delete(&mut self, key: &[u8]) -> Result<bool, Error> {
        validate_key(key)?;
        match self.kv.invalidate_key(hash_key(key)) {
            Ok(_) => Ok(true),
            Err(ErrorCode::KeyNotFound) => Ok(false),
            Err(code) => Err(map_tickv(code)),
        }
    }

    /// Reclaims regions whose objects are all invalid.
    pub fn gc(&mut self) -> Result<usize, Error> {
        self.kv.garbage_collect().map_err(map_tickv)
    }
}

impl<F: NorFlash + ReadStorage> StorageTrait for TicKvStorage<'_, F> {
    type Error = Error;

    fn get(&mut self, key: &[u8], value: &mut [u8]) -> Result<usize, Self::Error> {
        TicKvStorage::get(self, key, value)
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        TicKvStorage::set(self, key, value)
    }

    fn delete(&mut self, key: &[u8]) -> Result<bool, Self::Error> {
        TicKvStorage::delete(self, key)
    }
}

fn validate_key(key: &[u8]) -> Result<(), Error> {
    if key.is_empty() || key.len() > MAX_KEY_LEN {
        return Err(Error::InvalidKey);
    }
    Ok(())
}

const fn map_tickv(code: ErrorCode) -> Error {
    match code {
        ErrorCode::KeyNotFound => Error::NotFound,
        ErrorCode::KeyAlreadyExists => Error::Corrupt,
        ErrorCode::RegionFull | ErrorCode::FlashFull => Error::Full,
        ErrorCode::CorruptData | ErrorCode::InvalidCheckSum | ErrorCode::UnsupportedVersion => {
            Error::Corrupt
        }
        ErrorCode::ObjectTooLarge => Error::ValueTooLarge,
        ErrorCode::BufferTooSmall(needed) => Error::BufferTooSmall(needed),
        ErrorCode::ReadFail | ErrorCode::WriteFail | ErrorCode::EraseFail => Error::Flash,
        ErrorCode::ReadNotReady(_) | ErrorCode::WriteNotReady(_) | ErrorCode::EraseNotReady(_) => {
            Error::NotReady
        }
    }
}
