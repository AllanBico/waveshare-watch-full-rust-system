//! Sleep policy handed to the firmware for one deep-sleep cycle.

use super::plan::RailPlan;

/// What the firmware should do when putting the SoC into deep sleep.
///
/// `power` owns the policy; `firmware` translates it into the SoC's RTC wake
/// sources and calls `sleep_deep`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SleepPlan {
    /// Wake from the internal RTC timer after this many milliseconds, when an
    /// alarm is set.
    pub timer_ms: Option<u32>,
    /// Enable the power-button GPIO as a wake source.
    pub wake_on_button: bool,
    /// Rails to leave enabled during sleep.
    pub rails: RailPlan,
}
