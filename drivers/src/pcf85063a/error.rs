//! PCF85063A driver errors.

use core::fmt;

use common::Error;

/// Error returned by the PCF85063A driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum RtcError<E> {
    /// The I2C bus transaction failed.
    Bus(E),
    /// A date/time field was out of range or unsupported.
    InvalidField,
    /// The oscillator stopped, so the stored time is not valid.
    TimeInvalid,
}

impl<E> RtcError<E> {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(&self) -> Error {
        match self {
            RtcError::Bus(_) => Error::HardwareFailure,
            RtcError::InvalidField => Error::InvalidInput,
            RtcError::TimeInvalid => Error::TimeInvalid,
        }
    }
}

impl<E> fmt::Display for RtcError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            RtcError::Bus(_) => "i2c bus error",
            RtcError::InvalidField => "invalid date/time field",
            RtcError::TimeInvalid => "rtc time invalid",
        })
    }
}
