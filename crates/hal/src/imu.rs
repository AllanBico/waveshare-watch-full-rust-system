//! Inertial measurement contract.
//!
//! A device-level interface an IMU driver fulfills and the input/wake layers
//! consume. Reports are scaled to milli-g and milli-degrees per second so
//! orientation and threshold logic live above the driver.

use common::{MotionSample, WakeOnMotion};

/// A readable, motion-wake-capable IMU.
pub trait Imu {
    /// Driver-specific error.
    type Error;

    /// Reads one consistent accelerometer and gyroscope snapshot.
    fn read_motion(&mut self) -> Result<MotionSample, Self::Error>;

    /// Enables Wake-on-Motion with INT1 initially low.
    fn configure_wake_on_motion(&mut self, config: WakeOnMotion) -> Result<(), Self::Error>;

    /// Reads and clears the Wake-on-Motion event flag.
    fn take_wake_event(&mut self) -> Result<bool, Self::Error>;
}
