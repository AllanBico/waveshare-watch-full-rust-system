//! Derived boot policy.
//!
//! The settings schema stores user intent. This module clamps that intent to
//! the device's safety limits before firmware applies it to wake and sleep.

use common::Percent;

use crate::Settings;

/// Hard floor for the watch idle timeout.
const MIN_WATCH_IDLE_MS: u32 = 5_000;
/// Hard ceiling for the watch hard awake cap.
const MAX_WATCH_AWAKE_MS: u32 = 3_600_000;
/// Hard floor for the watch hard awake cap.
const MIN_WATCH_AWAKE_MS: u32 = 10_000;
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
    /// Idle time after the last touch before the watch sleeps.
    pub idle_timeout_ms: u32,
    /// Hard ceiling for one awake window, even while touched.
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
        let awake_ms = u32::from(settings.power.awake_cap_secs)
            .saturating_mul(1_000)
            .clamp(MIN_WATCH_AWAKE_MS, MAX_WATCH_AWAKE_MS);
        let idle_ms = u32::from(settings.power.sleep_timeout_secs)
            .saturating_mul(1_000)
            .clamp(MIN_WATCH_IDLE_MS, awake_ms);
        Self {
            wake_on_motion: settings.power.wake_on_motion,
            motion_threshold_mg: threshold as u8,
            idle_timeout_ms: idle_ms,
            awake_budget_ms: awake_ms,
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
        assert_eq!(policy.idle_timeout_ms, 30_000);
        assert_eq!(policy.awake_budget_ms, 120_000);
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
    fn idle_and_cap_are_clamped() {
        let mut settings = Settings::new();
        settings.power.sleep_timeout_secs = 0;
        settings.power.awake_cap_secs = 0;
        let policy = BootPolicy::from_settings(&settings);
        assert_eq!(policy.idle_timeout_ms, MIN_WATCH_IDLE_MS);
        assert_eq!(policy.awake_budget_ms, MIN_WATCH_AWAKE_MS);

        settings.power.sleep_timeout_secs = u16::MAX;
        settings.power.awake_cap_secs = u16::MAX;
        let policy = BootPolicy::from_settings(&settings);
        assert_eq!(policy.awake_budget_ms, MAX_WATCH_AWAKE_MS);
        assert_eq!(policy.idle_timeout_ms, MAX_WATCH_AWAKE_MS);
    }

    #[test]
    fn idle_timeout_never_exceeds_the_cap() {
        let mut settings = Settings::new();
        settings.power.sleep_timeout_secs = 3_600;
        settings.power.awake_cap_secs = 60;
        let policy = BootPolicy::from_settings(&settings);
        assert_eq!(policy.awake_budget_ms, 60_000);
        assert_eq!(policy.idle_timeout_ms, 60_000);
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
