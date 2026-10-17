//! QMI8658C driver errors.

use core::fmt;

use common::Error;

/// Error returned by the QMI8658C driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ImuError<E> {
    /// The I2C bus transaction failed.
    Bus(E),
    /// The identity register did not report a QMI8658C.
    BadChipId(u8),
    /// The Wake-on-Motion configuration is outside the register limits.
    InvalidConfig,
    /// A CTRL9 command did not complete before the poll limit.
    Timeout,
}

impl<E> ImuError<E> {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(&self) -> Error {
        match self {
            ImuError::Bus(_) | ImuError::BadChipId(_) => Error::HardwareFailure,
            ImuError::InvalidConfig => Error::InvalidInput,
            ImuError::Timeout => Error::Timeout,
        }
    }
}

impl<E> fmt::Display for ImuError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImuError::Bus(_) => f.write_str("imu i2c bus error"),
            ImuError::BadChipId(id) => write!(f, "unexpected imu chip id 0x{id:02X}"),
            ImuError::InvalidConfig => f.write_str("invalid wake-on-motion configuration"),
            ImuError::Timeout => f.write_str("imu command timed out"),
        }
    }
}
