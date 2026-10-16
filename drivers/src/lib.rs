//! Chip-specific drivers.
//!
//! Implementations of the HAL traits for the CO5300 display, FT3168 touch,
//! QMI8658 IMU, AXP2101 PMIC, and ES8311 codec. Only this crate touches
//! registers, pins, and I2C addresses.

#![no_std]
