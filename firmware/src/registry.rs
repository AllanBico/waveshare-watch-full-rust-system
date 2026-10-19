//! Firmware-owned app registry.
//!
//! Phase 7.1 registers exactly one real app in both modes: Home. The watch app
//! set stays minimal; the desk registry diverges in Phase 8.3.

use apps::Home;
use common::{Mode, Percent, Timestamp};
use settings::TimeFormat;
use shell::{App, AppId, AppRegistry};

/// The pinned Home route for both modes.
pub const HOME_ID: AppId = AppId::new(1);

/// A fixed registry with one app in both modes.
pub struct Registry {
    home: Home,
}

impl Registry {
    /// Marks the alarm Home should show as ringing.
    pub fn set_alarm(&mut self, alarm: Option<Timestamp>) {
        self.home.set_alarm(alarm);
    }

    /// Creates the registry with a boot-time Home state.
    pub fn new(
        mode: Mode,
        now: Timestamp,
        battery: Percent,
        is_charging: bool,
        time_format: TimeFormat,
    ) -> Self {
        Self {
            home: Home::new(mode, now, battery, is_charging, time_format),
        }
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
