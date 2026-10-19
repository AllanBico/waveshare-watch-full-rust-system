//! Firmware-owned app registry.
//!
//! Phase 6.3 has exactly one app in both modes: a no-op debug Home. The
//! firmware presenter draws its mode label, because the shell and `App` trait
//! intentionally do not own rendering yet. The real app set starts in 7.1.
// TODO(phase-7.1): replace with the real Home app and the app view seam.

use shell::{App, AppId, AppRegistry};

/// The pinned Home route for both modes.
pub const HOME_ID: AppId = AppId::new(1);

/// A lifecycle placeholder for Home.
///
/// It has no state and no render method. The firmware paints the mode label
/// from shell state until the real Home exists.
pub struct DebugHome;

impl App for DebugHome {}

/// A fixed registry with one app in both modes.
pub struct Registry {
    home: DebugHome,
}

impl Registry {
    /// Creates the Phase 6.3 registry.
    pub const fn new() -> Self {
        Self { home: DebugHome }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl AppRegistry for Registry {
    fn with_app<R>(&mut self, id: AppId, f: impl FnOnce(&mut dyn App) -> R) -> Option<R> {
        if id == HOME_ID {
            Some(f(&mut self.home))
        } else {
            None
        }
    }
}
