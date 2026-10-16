//! Screens: clock, settings, logs, voice.
//!
//! Each app implements the shared `App` trait and owns its own state. Apps
//! never touch hardware and never branch on `Mode`.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
