//! ES8311 driver errors.

use core::fmt;

use common::Error;

/// Error returned by the ES8311 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Es8311Error<E> {
    /// The I2C bus transaction failed.
    Bus(E),
    /// The chip identity did not match the ES8311.
    BadChipId(u8),
    /// The volume percentage was greater than 100.
    InvalidVolume(u8),
}

impl<E> Es8311Error<E> {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(&self) -> Error {
        match self {
            Es8311Error::Bus(_) => Error::HardwareFailure,
            Es8311Error::BadChipId(_) => Error::HardwareFailure,
            Es8311Error::InvalidVolume(_) => Error::InvalidInput,
        }
    }
}

impl<E> fmt::Display for Es8311Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Es8311Error::Bus(_) => "i2c bus error",
            Es8311Error::BadChipId(_) => "unexpected es8311 chip id",
            Es8311Error::InvalidVolume(_) => "volume out of range",
        })
    }
}
