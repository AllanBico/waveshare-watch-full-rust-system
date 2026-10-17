//! Storage errors.

use core::fmt;

use common::Error as CommonError;

/// Error returned by the storage layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The key does not exist.
    NotFound,
    /// The store has no space left.
    Full,
    /// Stored data failed an integrity check.
    Corrupt,
    /// The flash reported a failure.
    Flash,
    /// The key is empty or longer than [`crate::MAX_KEY_LEN`].
    InvalidKey,
    /// The flash size is zero or not a multiple of the region size.
    InvalidRegion,
    /// The value is larger than [`crate::MAX_VALUE_LEN`].
    ValueTooLarge,
    /// The caller's buffer is too small. Holds the required length.
    BufferTooSmall(usize),
    /// The store is not provisioned or not ready.
    NotReady,
    /// The stored schema version does not match this firmware.
    VersionMismatch,
}

impl Error {
    /// Maps to a stable cross-cutting [`CommonError`] code.
    pub const fn to_common(self) -> CommonError {
        match self {
            Error::NotFound => CommonError::NotFound,
            Error::Full => CommonError::StorageFull,
            Error::Corrupt => CommonError::StorageCorrupt,
            Error::Flash => CommonError::HardwareFailure,
            Error::InvalidKey
            | Error::InvalidRegion
            | Error::ValueTooLarge
            | Error::BufferTooSmall(_) => CommonError::InvalidInput,
            Error::NotReady => CommonError::NotReady,
            Error::VersionMismatch => CommonError::ConfigInvalid,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotFound => f.write_str("not found"),
            Error::Full => f.write_str("storage full"),
            Error::Corrupt => f.write_str("storage corrupt"),
            Error::Flash => f.write_str("flash failure"),
            Error::InvalidKey => f.write_str("invalid key"),
            Error::InvalidRegion => f.write_str("invalid storage region"),
            Error::ValueTooLarge => f.write_str("value too large"),
            Error::BufferTooSmall(needed) => write!(f, "buffer too small, need {needed}"),
            Error::NotReady => f.write_str("storage not ready"),
            Error::VersionMismatch => f.write_str("storage version mismatch"),
        }
    }
}
