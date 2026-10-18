//! Shell-owned status values shared across apps.

use common::{Percent, Timestamp};

/// The persistent status model painted by the shell.
///
/// Apps publish battery changes through [`crate::AppContext`]; the shell owns
/// the clock and keeps status alive across app and route changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusModel {
    clock: Timestamp,
    battery: Percent,
    is_charging: bool,
}

impl StatusModel {
    /// Creates an empty status model.
    pub const fn new() -> Self {
        Self {
            clock: Timestamp::from_millis(0),
            battery: Percent::new(0),
            is_charging: false,
        }
    }

    /// The current wall-clock time.
    pub const fn clock(&self) -> Timestamp {
        self.clock
    }

    /// The last published battery level.
    pub const fn battery(&self) -> Percent {
        self.battery
    }

    /// Whether the battery is charging.
    pub const fn is_charging(&self) -> bool {
        self.is_charging
    }

    /// Updates the clock. Only the shell calls this.
    pub fn set_clock(&mut self, clock: Timestamp) {
        self.clock = clock;
    }

    /// Updates the battery state.
    pub fn set_battery(&mut self, battery: Percent, is_charging: bool) {
        self.battery = battery;
        self.is_charging = is_charging;
    }
}

impl Default for StatusModel {
    fn default() -> Self {
        Self::new()
    }
}
