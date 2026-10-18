//! Settings errors.

use core::fmt;

/// A settings validation or persistence failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// The value does not match the setting's type.
    WrongType,
    /// The value is outside the setting's allowed range.
    OutOfRange,
    /// The encoded value is malformed.
    InvalidEncoding,
    /// The caller's output buffer is too small.
    BufferTooSmall,
    /// The backend rejected a write.
    Storage,
    /// No backend is available, so nothing can be written.
    Unavailable,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WrongType => formatter.write_str("wrong setting type"),
            Self::OutOfRange => formatter.write_str("setting value out of range"),
            Self::InvalidEncoding => formatter.write_str("invalid setting encoding"),
            Self::BufferTooSmall => formatter.write_str("setting buffer too small"),
            Self::Storage => formatter.write_str("settings storage failure"),
            Self::Unavailable => formatter.write_str("settings storage unavailable"),
        }
    }
}
