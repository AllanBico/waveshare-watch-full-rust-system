//! Derived boot policy.
//!
//! The settings schema stores user intent. This module clamps that intent to
//! the device's safety limits before firmware applies it to wake and sleep.

use common::Percent;

use crate::Settings;

/// Hard floor for the watch awake budget.
const MIN_WATCH_AWAKE_MS: u32 = 1_000;
/// Hard ceiling for the watch awake budget.
const MAX_WATCH_AWAKE_MS: u32 = 7_000;
/// Lowest Wake-on-Motion threshold in mg, the most sensitive setting.
const MIN_MOTION_THRESHOLD_MG: u8 = 32;
/// Highest Wake-on-Motion threshold in mg, the least sensitive setting.
const MAX_MOTION_THRESHOLD_MG: u8 = 128;

/// Settings-derived values applied at boot and before sleep.
///
/// Brightness and theme are live settings and are not part of this policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BootPolicy {
    /// Whether motion may wake the watch.
    pub wake_on_motion: bool,
    /// Clamped Wake-on-Motion threshold.
    pub motion_threshold_mg: u8,
    /// Clamped watch awake budget.
    pub awake_budget_ms: u32,
    /// Haptic strength for alarms.
    pub haptics: Percent,
}

impl BootPolicy {
    /// Derives the policy from a settings snapshot.
    pub fn from_settings(settings: &Settings) -> Self {
        let sensitivity = settings.power.motion_sensitivity.value().min(100);
        let range = u32::from(MAX_MOTION_THRESHOLD_MG - MIN_MOTION_THRESHOLD_MG);
        let threshold = u32::from(MAX_MOTION_THRESHOLD_MG) - u32::from(sensitivity) * range / 100;
        let awake_ms = u32::from(settings.power.sleep_timeout_secs).saturating_mul(1_000);
        Self {
            wake_on_motion: settings.power.wake_on_motion,
            motion_threshold_mg: threshold as u8,
            awake_budget_ms: awake_ms.clamp(MIN_WATCH_AWAKE_MS, MAX_WATCH_AWAKE_MS),
            haptics: settings.sound.haptics,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_map_to_bounded_values() {
        let policy = BootPolicy::from_settings(&Settings::new());
        assert!(policy.wake_on_motion);
        assert_eq!(policy.motion_threshold_mg, 80);
        assert_eq!(policy.awake_budget_ms, MAX_WATCH_AWAKE_MS);
        assert_eq!(policy.haptics.value(), 60);
    }

    #[test]
    fn sensitivity_maps_to_threshold_ends() {
        let mut settings = Settings::new();
        settings.power.motion_sensitivity = Percent::new(0);
        assert_eq!(
            BootPolicy::from_settings(&settings).motion_threshold_mg,
            MAX_MOTION_THRESHOLD_MG
        );
        settings.power.motion_sensitivity = Percent::new(100);
        assert_eq!(
            BootPolicy::from_settings(&settings).motion_threshold_mg,
            MIN_MOTION_THRESHOLD_MG
        );
    }

    #[test]
    fn awake_budget_is_clamped() {
        let mut settings = Settings::new();
        settings.power.sleep_timeout_secs = 0;
        assert_eq!(
            BootPolicy::from_settings(&settings).awake_budget_ms,
            MIN_WATCH_AWAKE_MS
        );
        settings.power.sleep_timeout_secs = u16::MAX;
        assert_eq!(
            BootPolicy::from_settings(&settings).awake_budget_ms,
            MAX_WATCH_AWAKE_MS
        );
    }

    #[test]
    fn wake_and_haptics_follow_settings() {
        let mut settings = Settings::new();
        settings.power.wake_on_motion = false;
        settings.sound.haptics = Percent::new(20);
        let policy = BootPolicy::from_settings(&settings);
        assert!(!policy.wake_on_motion);
        assert_eq!(policy.haptics.value(), 20);
    }
}
