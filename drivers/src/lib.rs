//! Chip-specific drivers.
//!
//! Implementations of the HAL traits for the CO5300 display, FT3168 touch,
//! QMI8658 IMU, AXP2101 PMIC, PCF85063A RTC, and ES8311 codec. The single
//! pin map lives in [`board`]. Only this crate touches registers, pins, and
//! I2C addresses.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

pub mod axp2101;
pub mod board;
pub mod co5300;
#[cfg(feature = "hardware")]
pub mod flash;
pub mod ft3168;
pub mod pcf85063a;
pub mod qmi8658;

#[cfg(test)]
mod test_support;
