//! Wall-clock abstraction.

use common::Timestamp;

/// A readable and settable wall clock (UTC).
pub trait Time {
    /// Driver-specific error.
    type Error;

    /// Reads the current time.
    ///
    /// Returns an error rather than a bogus value if the clock reports its
    /// time as invalid (e.g. oscillator stopped).
    fn now(&mut self) -> Result<Timestamp, Self::Error>;

    /// Sets the current time.
    fn set(&mut self, timestamp: Timestamp) -> Result<(), Self::Error>;
}
