//! Screens for the in-house UI.
//!
//! Apps own their state and produce pure views through the fixed-capacity UI
//! engine. They never touch hardware and never branch on `Mode` inside widgets.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod audio_lab;
mod format;
pub mod home;
mod logs;
mod overlays;
mod settings_app;
mod view;

use shell::AppId;

pub use audio_lab::{AudioLab, RenderError as AudioLabRenderError};
pub use home::{Home, HomeState, RenderError};
pub use logs::{LogCategory, LogsApp};
pub use overlays::{Drawer, QuickSettings};
pub use settings_app::SettingsApp;

/// Stable app id for Settings.
pub const SETTINGS_APP_ID: AppId = AppId::new(2);
/// Stable app id for Logs.
pub const LOGS_APP_ID: AppId = AppId::new(3);
/// Stable app id for the desk Audio Lab.
pub const AUDIO_LAB_APP_ID: AppId = AppId::new(4);
