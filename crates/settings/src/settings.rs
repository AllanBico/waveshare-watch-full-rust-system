//! The typed settings snapshot.

use common::{Mode, Percent, ThemeId};

use crate::value::{DateFormat, DeviceName, FirstDay, TimeFormat, Units};

/// Display and appearance settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisplaySettings {
    /// Brightness in watch mode.
    pub brightness_watch: Percent,
    /// Brightness in desk mode.
    pub brightness_desk: Percent,
    /// Clock format.
    pub time_format: TimeFormat,
    /// Date order.
    pub date_format: DateFormat,
    /// Measurement units.
    pub units: Units,
    /// First day of the week.
    pub first_day: FirstDay,
    /// Theme variant.
    pub theme: ThemeId,
    /// Screen timeout in seconds.
    pub screen_timeout_secs: u16,
    /// Whether the screen stays on.
    pub always_on: bool,
}

/// Sound and haptics settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SoundSettings {
    /// Output volume.
    pub volume: Percent,
    /// Whether audio output is muted.
    pub is_muted: bool,
    /// Whether notifications are silenced.
    pub is_dnd: bool,
    /// Haptic strength.
    pub haptics: Percent,
    /// Wake-word sensitivity.
    pub wake_sensitivity: Percent,
}

/// Radio and sync settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConnectivitySettings {
    /// Whether Wi-Fi may be enabled.
    pub wifi: bool,
    /// Whether Bluetooth may be enabled.
    pub bluetooth: bool,
    /// Whether background sync may run.
    pub sync: bool,
    /// Delay between sync attempts, in seconds.
    pub auto_sync_secs: u16,
}

/// Power and wake settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSettings {
    /// Whether motion may wake the watch.
    pub wake_on_motion: bool,
    /// Motion wake sensitivity.
    pub motion_sensitivity: Percent,
    /// Idle time before sleep, in seconds.
    pub sleep_timeout_secs: u16,
    /// Whether aggressive power saving is enabled.
    pub power_saving: bool,
}

/// Device identity settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SystemSettings {
    /// Offset from UTC, in minutes.
    pub timezone_offset_minutes: i16,
    /// User-visible device name.
    pub device_name: DeviceName,
}

/// Every persisted runtime setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Settings {
    /// Display and appearance.
    pub display: DisplaySettings,
    /// Sound and haptics.
    pub sound: SoundSettings,
    /// Radios and sync.
    pub connectivity: ConnectivitySettings,
    /// Power and wake behavior.
    pub power: PowerSettings,
    /// Device identity.
    pub system: SystemSettings,
}

impl Settings {
    /// Defaults for a fresh device.
    pub const fn new() -> Self {
        Self {
            display: DisplaySettings {
                brightness_watch: Percent::new(70),
                brightness_desk: Percent::new(70),
                time_format: TimeFormat::Hour24,
                date_format: DateFormat::MonthDay,
                units: Units::Metric,
                first_day: FirstDay::Sunday,
                theme: ThemeId::Default,
                screen_timeout_secs: 15,
                always_on: false,
            },
            sound: SoundSettings {
                volume: Percent::new(60),
                is_muted: false,
                is_dnd: false,
                haptics: Percent::new(60),
                wake_sensitivity: Percent::new(50),
            },
            connectivity: ConnectivitySettings {
                wifi: true,
                bluetooth: false,
                sync: true,
                auto_sync_secs: 300,
            },
            power: PowerSettings {
                wake_on_motion: true,
                motion_sensitivity: Percent::new(50),
                sleep_timeout_secs: 30,
                power_saving: false,
            },
            system: SystemSettings {
                timezone_offset_minutes: 0,
                device_name: DeviceName::new(),
            },
        }
    }

    /// Brightness for the active mode.
    pub const fn brightness(&self, mode: Mode) -> Percent {
        match mode {
            Mode::Watch => self.display.brightness_watch,
            Mode::Desk => self.display.brightness_desk,
        }
    }

    /// Sets the brightness for the active mode.
    pub fn set_brightness(&mut self, mode: Mode, brightness: Percent) {
        match mode {
            Mode::Watch => self.display.brightness_watch = brightness,
            Mode::Desk => self.display.brightness_desk = brightness,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sane() {
        let settings = Settings::new();
        assert_eq!(settings.brightness(Mode::Watch).value(), 70);
        assert_eq!(settings.brightness(Mode::Desk).value(), 70);
        assert_eq!(settings.display.time_format, TimeFormat::Hour24);
        assert!(settings.connectivity.wifi);
        assert!(settings.power.wake_on_motion);
        assert_eq!(settings.system.device_name.as_str(), "wv2");
    }

    #[test]
    fn brightness_is_per_mode() {
        let mut settings = Settings::new();
        settings.set_brightness(Mode::Watch, Percent::new(20));
        assert_eq!(settings.display.brightness_watch.value(), 20);
        assert_eq!(settings.display.brightness_desk.value(), 70);
    }
}
