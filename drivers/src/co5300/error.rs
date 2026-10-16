//! CO5300 driver errors.

use core::fmt;

use common::Error;

/// Error returned by the CO5300 driver.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum DisplayError<E> {
    /// The transport failed.
    Bus(E),
    /// The reset pin could not be driven.
    Reset,
    /// The region is empty, inverted, or outside the panel.
    RegionOutOfBounds,
    /// The window start or extent is not an even number of pixels.
    RegionMisaligned,
    /// The pixel byte count does not match the region.
    LengthMismatch,
}

impl<E> DisplayError<E> {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(&self) -> Error {
        match self {
            DisplayError::Bus(_) => Error::HardwareFailure,
            DisplayError::Reset => Error::HardwareFailure,
            DisplayError::RegionOutOfBounds
            | DisplayError::RegionMisaligned
            | DisplayError::LengthMismatch => Error::InvalidInput,
        }
    }
}

impl<E> fmt::Display for DisplayError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            DisplayError::Bus(_) => "display transport error",
            DisplayError::Reset => "display reset failed",
            DisplayError::RegionOutOfBounds => "region out of bounds",
            DisplayError::RegionMisaligned => "region is not 2-pixel aligned",
            DisplayError::LengthMismatch => "pixel length mismatch",
        })
    }
}
