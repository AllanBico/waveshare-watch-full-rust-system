//! Chip-specific drivers.
//!
//! Implementations of the HAL traits for the CO5300 display, FT3168 touch,
//! QMI8658 IMU, AXP2101 PMIC, PCF85063A RTC, and ES8311 codec. The single
//! pin map will live in a `board` module. Only this crate touches registers,
//! pins, and I2C addresses.

#![cfg_attr(not(test), no_std)]
