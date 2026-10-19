//! Shell-owned status values shared across apps.

use common::{Percent, SystemStats, Timestamp};

/// The persistent status model painted by the shell.
///
/// Apps publish battery changes through [`crate::AppContext`]; the shell owns
/// the clock, system stats, and keeps status alive across app and route
/// changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StatusModel {
    clock: Timestamp,
    battery: Percent,
    is_charging: bool,
    system_stats: SystemStats,
}

impl StatusModel {
    /// Creates an empty status model.
    pub const fn new() -> Self {
        Self {
            clock: Timestamp::from_millis(0),
            battery: Percent::new(0),
            is_charging: false,
            system_stats: SystemStats::empty(),
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

    /// The last system stats sample.
    pub const fn system_stats(&self) -> SystemStats {
        self.system_stats
    }

    /// Updates the system stats sample.
    pub fn set_system_stats(&mut self, stats: SystemStats) {
        self.system_stats = stats;
    }
}

impl Default for StatusModel {
    fn default() -> Self {
        Self::new()
    }
}
