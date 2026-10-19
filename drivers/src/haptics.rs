//! Vibration-motor low-side switch.
//!
//! The motor supply is ALDO3, already enabled by the watch rail plan. GPIO18
//! drives the low-side transistor; keep it low unless an alarm is ringing.

use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::GPIO18;

/// The board's vibration motor.
pub struct Motor {
    pin: Output<'static>,
}

impl Motor {
    /// Creates the motor output, held off.
    pub fn new(pin: GPIO18<'static>) -> Self {
        Self {
            pin: Output::new(pin, Level::Low, OutputConfig::default()),
        }
    }

    /// Drives the motor on or off.
    pub fn set(&mut self, is_on: bool) {
        if is_on {
            self.pin.set_high();
        } else {
            self.pin.set_low();
        }
    }
}
