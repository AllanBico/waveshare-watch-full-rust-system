//! AXP2101 driver errors.

use core::fmt;

/// A value that a rail or the system-voltage register cannot represent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum RailError {
    /// The requested value is outside every valid segment.
    OutOfRange,
    /// The value is in range but not on a voltage step.
    OffStep,
    /// The operation does not apply to this rail.
    NotApplicable,
}

/// Error returned by the AXP2101 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum AxpError<E> {
    /// The I2C bus transaction failed.
    Bus(E),
    /// The chip identity did not match the AXP2101.
    BadChipId(u8),
    /// A rail or system-voltage value was rejected.
    Voltage(RailError),
}

impl<E> From<RailError> for AxpError<E> {
    fn from(error: RailError) -> Self {
        AxpError::Voltage(error)
    }
}

impl<E> fmt::Display for AxpError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            AxpError::Bus(_) => "i2c bus error",
            AxpError::BadChipId(_) => "unexpected chip id",
            AxpError::Voltage(RailError::OutOfRange) => "voltage out of range",
            AxpError::Voltage(RailError::OffStep) => "voltage not on a step",
            AxpError::Voltage(RailError::NotApplicable) => "operation not applicable",
        })
    }
}
