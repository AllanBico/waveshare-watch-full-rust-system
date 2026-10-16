//! Firmware entry point.
//!
//! One binary for both modes. This crate owns initialization and the mode
//! state machine; it reads VBUS and dispatches to `watch_init` or `desk_init`.
//! It becomes a binary in Phase 0.2 when `esp-hal` is introduced.

#![no_std]

/// Bring up watch mode and enter the deep sleep loop.
///
/// Placeholder until the watch mode integration phase.
pub fn watch_init() {}

/// Bring up desk mode and start the full subsystem set.
///
/// Placeholder until the desk mode foundation phase.
pub fn desk_init() {}
