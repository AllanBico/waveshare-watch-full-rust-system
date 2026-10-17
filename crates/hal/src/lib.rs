//! Hardware abstraction traits.
//!
//! Device-level contracts only: `Display`, `Touch`, `IMU`, `Power`,
//! `Storage`, `Audio`, `Time`. No implementations live here.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod audio;
mod audio_capture;
mod display;
mod imu;
mod power;
mod storage;
mod time;
mod touch;

pub use audio::Audio;
pub use audio_capture::AudioCapture;
pub use display::Display;
pub use imu::Imu;
pub use power::{Power, Rail, RAIL_COUNT};
pub use storage::Storage;
pub use time::Time;
pub use touch::Touch;
