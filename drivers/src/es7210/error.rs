//! ES7210 driver errors.

use core::fmt;

use common::Error;

/// Error returned by the ES7210 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Es7210Error<E> {
    /// The I2C bus transaction failed.
    Bus(E),
    /// The microphone gain was not a supported 3 dB step from 0 to 30.
    InvalidGain(u8),
}

impl<E> Es7210Error<E> {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(&self) -> Error {
        match self {
            Es7210Error::Bus(_) => Error::HardwareFailure,
            Es7210Error::InvalidGain(_) => Error::InvalidInput,
        }
    }
}

impl<E> fmt::Display for Es7210Error<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Es7210Error::Bus(_) => "i2c bus error",
            Es7210Error::InvalidGain(_) => "unsupported microphone gain",
        })
    }
}
