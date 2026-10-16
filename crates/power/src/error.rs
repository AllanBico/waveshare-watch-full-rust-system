//! Power-manager control errors.

use common::Error;

/// Error returned by the power manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum PowerError<E> {
    /// The underlying PMIC failed.
    Pmic(E),
    /// A rail did not reach its planned state.
    AuditFailed,
    /// The subsystem was used before it was initialized.
    NotReady,
}

impl<E> PowerError<E> {
    /// Maps to a stable cross-cutting [`Error`] code.
    pub const fn to_common(&self) -> Error {
        match self {
            PowerError::Pmic(_) => Error::HardwareFailure,
            PowerError::AuditFailed => Error::PowerFail,
            PowerError::NotReady => Error::NotReady,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_to_common_codes() {
        assert_eq!(PowerError::<()>::AuditFailed.to_common(), Error::PowerFail);
        assert_eq!(PowerError::Pmic(()).to_common(), Error::HardwareFailure);
    }
}
