//! Screens for the in-house UI.
//!
//! Apps own their state and produce pure views through the fixed-capacity UI
//! engine. They never touch hardware and never branch on `Mode` inside widgets.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod format;
// Home and view are retained for the Phase 6.3 shell boot, not wired yet.
#[allow(dead_code)]
pub mod home;
#[allow(dead_code)]
mod view;

pub use home::HomeState;
