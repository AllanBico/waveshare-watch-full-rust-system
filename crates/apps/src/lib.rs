//! Screens for the in-house UI.
//!
//! Apps own their state and produce pure views through the fixed-capacity UI
//! engine. They never touch hardware and never branch on `Mode` inside widgets.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod format;
pub mod home;
mod overlays;
mod settings_app;
mod view;

use shell::AppId;

pub use home::{Home, HomeState, RenderError};
pub use overlays::{Drawer, QuickSettings};
pub use settings_app::SettingsApp;

/// Stable app id for Settings.
pub const SETTINGS_APP_ID: AppId = AppId::new(2);
