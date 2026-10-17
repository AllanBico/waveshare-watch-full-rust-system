//! Touch input contract.
//!
//! A device-level interface a touch driver fulfills and the input layer
//! consumes. Reports carry panel coordinates plus the controller's touch ID;
//! gesture interpretation belongs above this trait.

use common::TouchReport;

/// A readable, power-manageable touch controller.
pub trait Touch {
    /// Driver-specific error.
    type Error;

    /// Reads a consistent snapshot of the active touch points.
    fn read(&mut self) -> Result<TouchReport, Self::Error>;

    /// Puts the controller into its lowest-power scanning mode.
    fn sleep(&mut self) -> Result<(), Self::Error>;

    /// Restores active scanning after [`Self::sleep`].
    fn wake(&mut self) -> Result<(), Self::Error>;
}
