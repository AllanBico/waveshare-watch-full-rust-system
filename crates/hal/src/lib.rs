//! Hardware abstraction traits.
//!
//! Device-level contracts only: `Display`, `Touch`, `IMU`, `Power`,
//! `Storage`, `Audio`. No implementations live here.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
