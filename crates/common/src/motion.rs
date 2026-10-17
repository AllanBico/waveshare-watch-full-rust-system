//! Inertial input vocabulary.

/// A three-axis acceleration sample in milli-g.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Acceleration {
    /// X-axis acceleration.
    pub x_mg: i16,
    /// Y-axis acceleration.
    pub y_mg: i16,
    /// Z-axis acceleration.
    pub z_mg: i16,
}

impl Acceleration {
    /// A zero sample.
    pub const ZERO: Self = Self {
        x_mg: 0,
        y_mg: 0,
        z_mg: 0,
    };

    /// Creates a sample.
    pub const fn new(x_mg: i16, y_mg: i16, z_mg: i16) -> Self {
        Self { x_mg, y_mg, z_mg }
    }
}

/// A three-axis angular-rate sample in milli-degrees per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AngularRate {
    /// X-axis angular rate.
    pub x_mdps: i32,
    /// Y-axis angular rate.
    pub y_mdps: i32,
    /// Z-axis angular rate.
    pub z_mdps: i32,
}

impl AngularRate {
    /// A zero sample.
    pub const ZERO: Self = Self {
        x_mdps: 0,
        y_mdps: 0,
        z_mdps: 0,
    };

    /// Creates a sample.
    pub const fn new(x_mdps: i32, y_mdps: i32, z_mdps: i32) -> Self {
        Self {
            x_mdps,
            y_mdps,
            z_mdps,
        }
    }
}

/// A consistent accelerometer and gyroscope snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MotionSample {
    /// Linear acceleration.
    pub acceleration: Acceleration,
    /// Angular rate.
    pub angular_rate: AngularRate,
}

impl MotionSample {
    /// A zero sample.
    pub const ZERO: Self = Self {
        acceleration: Acceleration::ZERO,
        angular_rate: AngularRate::ZERO,
    };
}

/// Wake-on-Motion threshold and interrupt blanking.
///
/// The QMI8658 compares against an 8-bit threshold in mg and supports up to
/// 63 blanking samples. The interrupt line is INT1, initially low.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakeOnMotion {
    /// Threshold in mg, `1..=255`.
    pub threshold_mg: u8,
    /// Accelerometer samples ignored after enabling, `0..=63`.
    pub blanking_samples: u8,
}

impl WakeOnMotion {
    /// The largest threshold the sensor accepts.
    pub const MAX_THRESHOLD_MG: u8 = 255;
    /// The largest blanking count the sensor accepts.
    pub const MAX_BLANKING_SAMPLES: u8 = 63;

    /// Creates a configuration without validating it.
    pub const fn new(threshold_mg: u8, blanking_samples: u8) -> Self {
        Self {
            threshold_mg,
            blanking_samples,
        }
    }

    /// Whether the fields fit the sensor's register limits.
    pub const fn is_valid(&self) -> bool {
        self.threshold_mg > 0 && self.blanking_samples <= Self::MAX_BLANKING_SAMPLES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_samples_are_zero() {
        assert_eq!(Acceleration::ZERO, Acceleration::new(0, 0, 0));
        assert_eq!(AngularRate::ZERO, AngularRate::new(0, 0, 0));
        assert_eq!(MotionSample::ZERO.acceleration, Acceleration::ZERO);
        assert_eq!(MotionSample::ZERO.angular_rate, AngularRate::ZERO);
    }

    #[test]
    fn wake_on_motion_limits_are_enforced() {
        assert!(WakeOnMotion::new(1, 0).is_valid());
        assert!(WakeOnMotion::new(255, 63).is_valid());
        assert!(!WakeOnMotion::new(0, 0).is_valid());
        assert!(!WakeOnMotion::new(100, 64).is_valid());
    }
}
