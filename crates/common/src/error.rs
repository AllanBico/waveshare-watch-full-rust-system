//! Cross-cutting error type.

use core::fmt;

/// Cross-cutting error shared across crates.
///
/// Codes are stable and append-only: never renumber an existing variant. Each
/// module maps its own error type into this at its boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Error {
    /// Power rail or PMIC failure.
    PowerFail,
    /// Storage has no space left.
    StorageFull,
    /// Stored data failed an integrity check.
    StorageCorrupt,
    /// Configuration failed validation on load.
    ConfigInvalid,
    /// A peripheral did not respond as expected.
    HardwareFailure,
    /// An operation exceeded its time budget.
    Timeout,
    /// The requested capability is not available.
    Unsupported,
    /// Input failed validation at the boundary.
    InvalidInput,
    /// The network or VPS is unreachable.
    NetworkUnavailable,
    /// A subsystem was used before it was initialized.
    NotReady,
}

impl Error {
    /// Stable numeric code for logging and the wire contract.
    pub const fn code(self) -> u16 {
        match self {
            Error::PowerFail => 1,
            Error::StorageFull => 2,
            Error::StorageCorrupt => 3,
            Error::ConfigInvalid => 4,
            Error::HardwareFailure => 5,
            Error::Timeout => 6,
            Error::Unsupported => 7,
            Error::InvalidInput => 8,
            Error::NetworkUnavailable => 9,
            Error::NotReady => 10,
        }
    }

    /// Short human-readable description.
    pub const fn as_str(self) -> &'static str {
        match self {
            Error::PowerFail => "power failure",
            Error::StorageFull => "storage full",
            Error::StorageCorrupt => "storage corrupt",
            Error::ConfigInvalid => "invalid configuration",
            Error::HardwareFailure => "hardware failure",
            Error::Timeout => "operation timed out",
            Error::Unsupported => "unsupported operation",
            Error::InvalidInput => "invalid input",
            Error::NetworkUnavailable => "network unavailable",
            Error::NotReady => "subsystem not ready",
        }
    }

    /// Every variant, for exhaustiveness tests.
    #[cfg(test)]
    const ALL: [Error; 10] = [
        Error::PowerFail,
        Error::StorageFull,
        Error::StorageCorrupt,
        Error::ConfigInvalid,
        Error::HardwareFailure,
        Error::Timeout,
        Error::Unsupported,
        Error::InvalidInput,
        Error::NetworkUnavailable,
        Error::NotReady,
    ];
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable() {
        assert_eq!(Error::PowerFail.code(), 1);
        assert_eq!(Error::StorageFull.code(), 2);
        assert_eq!(Error::NotReady.code(), 10);
    }

    #[test]
    fn codes_are_unique() {
        for (i, left) in Error::ALL.iter().enumerate() {
            for right in &Error::ALL[i + 1..] {
                assert_ne!(left.code(), right.code(), "duplicate code");
            }
        }
    }

    #[test]
    fn every_variant_has_a_message() {
        for error in Error::ALL {
            assert!(!error.as_str().is_empty());
            assert_eq!(error.to_string(), error.as_str());
        }
    }
}
