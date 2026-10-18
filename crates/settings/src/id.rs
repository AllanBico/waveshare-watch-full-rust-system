//! Stable setting identifiers, keys, and schema metadata.

use common::{Mode, ThemeId};

use crate::value::ValueKind;
use crate::Settings;

/// Number of persisted settings.
pub const SETTING_COUNT: usize = 24;

/// The settings group a key belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SettingGroup {
    /// Display and appearance.
    Display,
    /// Sound and haptics.
    Sound,
    /// Radios and sync.
    Connectivity,
    /// Power and wake behavior.
    Power,
    /// Device identity.
    System,
}

/// A settings change category consumers can react to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SettingsEvent {
    /// The selected theme changed.
    Theme,
    /// Display brightness changed.
    Brightness,
    /// Another display setting changed.
    Display,
    /// A sound setting changed.
    Sound,
    /// A connectivity setting changed.
    Connectivity,
    /// A power setting changed.
    Power,
    /// A system setting changed.
    System,
}

/// One persisted setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum SettingId {
    /// Watch-mode brightness.
    DisplayBrightnessWatch = 0,
    /// Desk-mode brightness.
    DisplayBrightnessDesk = 1,
    /// Clock format.
    DisplayTimeFormat = 2,
    /// Date order.
    DisplayDateFormat = 3,
    /// Measurement units.
    DisplayUnits = 4,
    /// First day of the week.
    DisplayFirstDay = 5,
    /// Theme variant.
    DisplayTheme = 6,
    /// Screen timeout.
    DisplayScreenTimeout = 7,
    /// Always-on display.
    DisplayAlwaysOn = 8,
    /// Output volume.
    SoundVolume = 9,
    /// Mute toggle.
    SoundMuted = 10,
    /// Do-not-disturb toggle.
    SoundDnd = 11,
    /// Haptic strength.
    SoundHaptics = 12,
    /// Wake-word sensitivity.
    SoundWakeSensitivity = 13,
    /// Wi-Fi toggle.
    ConnectivityWifi = 14,
    /// Bluetooth toggle.
    ConnectivityBluetooth = 15,
    /// Sync toggle.
    ConnectivitySync = 16,
    /// Sync interval.
    ConnectivityAutoSync = 17,
    /// Motion wake toggle.
    PowerWakeOnMotion = 18,
    /// Motion wake sensitivity.
    PowerMotionSensitivity = 19,
    /// Sleep timeout.
    PowerSleepTimeout = 20,
    /// Power-saving toggle.
    PowerPowerSaving = 21,
    /// Timezone offset.
    SystemTimezoneOffset = 22,
    /// Device name.
    SystemDeviceName = 23,
}

impl SettingId {
    /// Every setting, in stable index order.
    pub const ALL: [Self; SETTING_COUNT] = [
        Self::DisplayBrightnessWatch,
        Self::DisplayBrightnessDesk,
        Self::DisplayTimeFormat,
        Self::DisplayDateFormat,
        Self::DisplayUnits,
        Self::DisplayFirstDay,
        Self::DisplayTheme,
        Self::DisplayScreenTimeout,
        Self::DisplayAlwaysOn,
        Self::SoundVolume,
        Self::SoundMuted,
        Self::SoundDnd,
        Self::SoundHaptics,
        Self::SoundWakeSensitivity,
        Self::ConnectivityWifi,
        Self::ConnectivityBluetooth,
        Self::ConnectivitySync,
        Self::ConnectivityAutoSync,
        Self::PowerWakeOnMotion,
        Self::PowerMotionSensitivity,
        Self::PowerSleepTimeout,
        Self::PowerPowerSaving,
        Self::SystemTimezoneOffset,
        Self::SystemDeviceName,
    ];

    /// Stable dense index.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// The bit used by dirty and change masks.
    pub const fn bit(self) -> u32 {
        1 << self.index()
    }

    /// The group this setting belongs to.
    pub const fn group(self) -> SettingGroup {
        match self {
            Self::DisplayBrightnessWatch
            | Self::DisplayBrightnessDesk
            | Self::DisplayTimeFormat
            | Self::DisplayDateFormat
            | Self::DisplayUnits
            | Self::DisplayFirstDay
            | Self::DisplayTheme
            | Self::DisplayScreenTimeout
            | Self::DisplayAlwaysOn => SettingGroup::Display,
            Self::SoundVolume
            | Self::SoundMuted
            | Self::SoundDnd
            | Self::SoundHaptics
            | Self::SoundWakeSensitivity => SettingGroup::Sound,
            Self::ConnectivityWifi
            | Self::ConnectivityBluetooth
            | Self::ConnectivitySync
            | Self::ConnectivityAutoSync => SettingGroup::Connectivity,
            Self::PowerWakeOnMotion
            | Self::PowerMotionSensitivity
            | Self::PowerSleepTimeout
            | Self::PowerPowerSaving => SettingGroup::Power,
            Self::SystemTimezoneOffset | Self::SystemDeviceName => SettingGroup::System,
        }
    }

    /// The change event this setting produces.
    pub const fn event(self) -> SettingsEvent {
        match self {
            Self::DisplayTheme => SettingsEvent::Theme,
            Self::DisplayBrightnessWatch | Self::DisplayBrightnessDesk => SettingsEvent::Brightness,
            id => match id.group() {
                SettingGroup::Display => SettingsEvent::Display,
                SettingGroup::Sound => SettingsEvent::Sound,
                SettingGroup::Connectivity => SettingsEvent::Connectivity,
                SettingGroup::Power => SettingsEvent::Power,
                SettingGroup::System => SettingsEvent::System,
            },
        }
    }

    /// The value type this setting expects.
    pub const fn kind(self) -> ValueKind {
        match self {
            Self::DisplayBrightnessWatch | Self::DisplayBrightnessDesk => ValueKind::Percent,
            Self::DisplayTimeFormat => ValueKind::TimeFormat,
            Self::DisplayDateFormat => ValueKind::DateFormat,
            Self::DisplayUnits => ValueKind::Units,
            Self::DisplayFirstDay => ValueKind::FirstDay,
            Self::DisplayTheme => ValueKind::Theme,
            Self::DisplayScreenTimeout => ValueKind::Seconds,
            Self::DisplayAlwaysOn => ValueKind::Bool,
            Self::SoundVolume | Self::SoundHaptics | Self::SoundWakeSensitivity => {
                ValueKind::Percent
            }
            Self::SoundMuted | Self::SoundDnd => ValueKind::Bool,
            Self::ConnectivityWifi | Self::ConnectivityBluetooth | Self::ConnectivitySync => {
                ValueKind::Bool
            }
            Self::ConnectivityAutoSync => ValueKind::Seconds,
            Self::PowerWakeOnMotion | Self::PowerPowerSaving => ValueKind::Bool,
            Self::PowerMotionSensitivity => ValueKind::Percent,
            Self::PowerSleepTimeout => ValueKind::Seconds,
            Self::SystemTimezoneOffset => ValueKind::OffsetMinutes,
            Self::SystemDeviceName => ValueKind::DeviceName,
        }
    }

    /// The brightness setting for a mode.
    pub const fn brightness(mode: Mode) -> Self {
        match mode {
            Mode::Watch => Self::DisplayBrightnessWatch,
            Mode::Desk => Self::DisplayBrightnessDesk,
        }
    }

    /// The default value for this setting.
    pub fn default_value(self) -> crate::SettingValue {
        self.read(&Settings::new())
    }

    /// Whether two settings are both brightness keys.
    pub const fn is_brightness(self) -> bool {
        matches!(
            self,
            Self::DisplayBrightnessWatch | Self::DisplayBrightnessDesk
        )
    }

    /// The theme setting.
    pub const fn is_theme(self) -> bool {
        matches!(self, Self::DisplayTheme)
    }

    /// A stable short label used by diagnostics.
    pub const fn label(self) -> &'static str {
        match self {
            Self::DisplayBrightnessWatch => "brightness.watch",
            Self::DisplayBrightnessDesk => "brightness.desk",
            Self::DisplayTimeFormat => "time_format",
            Self::DisplayDateFormat => "date_format",
            Self::DisplayUnits => "units",
            Self::DisplayFirstDay => "first_day",
            Self::DisplayTheme => "theme",
            Self::DisplayScreenTimeout => "screen_timeout",
            Self::DisplayAlwaysOn => "always_on",
            Self::SoundVolume => "volume",
            Self::SoundMuted => "muted",
            Self::SoundDnd => "dnd",
            Self::SoundHaptics => "haptics",
            Self::SoundWakeSensitivity => "wake_sensitivity",
            Self::ConnectivityWifi => "wifi",
            Self::ConnectivityBluetooth => "bluetooth",
            Self::ConnectivitySync => "sync",
            Self::ConnectivityAutoSync => "auto_sync",
            Self::PowerWakeOnMotion => "wake_on_motion",
            Self::PowerMotionSensitivity => "motion_sensitivity",
            Self::PowerSleepTimeout => "sleep_timeout",
            Self::PowerPowerSaving => "power_saving",
            Self::SystemTimezoneOffset => "timezone_offset",
            Self::SystemDeviceName => "device_name",
        }
    }

    /// A default theme id, exposed to keep the default visible to consumers.
    pub const DEFAULT_THEME: ThemeId = ThemeId::Default;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_dense_and_keys_unique() {
        for (index, id) in SettingId::ALL.iter().enumerate() {
            assert_eq!(usize::from(id.index()), index);
        }
        for (index, id) in SettingId::ALL.iter().enumerate() {
            for other in &SettingId::ALL[index + 1..] {
                assert_ne!(id.key(), other.key());
            }
        }
    }
}
