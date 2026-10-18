//! Screens and preview views for the in-house UI.
//!
//! Apps own their state and produce pure views through the fixed-capacity UI
//! engine. They never touch hardware and never branch on `Mode` inside widgets.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod format;
pub mod home;
pub mod preview;
mod view;

pub use home::HomeState;
pub use preview::{Overlay, Preview, QuickState, RenderError, Screen};
