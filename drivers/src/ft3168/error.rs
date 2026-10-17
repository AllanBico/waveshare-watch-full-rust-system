//! FT3168 driver errors.

use core::fmt;

use common::Error;

/// Error returned by the FT3168 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum TouchError<E> {
    /// The I2C bus transaction failed.
    Bus(E),
    /// The reset pin could not be driven.
    Reset,
}

impl<E> TouchError<E> {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(&self) -> Error {
        match self {
            TouchError::Bus(_) | TouchError::Reset => Error::HardwareFailure,
        }
    }
}

impl<E> fmt::Display for TouchError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            TouchError::Bus(_) => "touch i2c bus error",
            TouchError::Reset => "touch reset failed",
        })
    }
}
