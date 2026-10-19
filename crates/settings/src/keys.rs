//! Persistence keys for every setting.

use super::SettingId;

impl SettingId {
    /// The persistence key. The `v1` segment is the schema version.
    pub const fn key(self) -> &'static [u8] {
        match self {
            Self::DisplayBrightnessWatch => b"settings/v1/display.brightness_watch",
            Self::DisplayBrightnessDesk => b"settings/v1/display.brightness_desk",
            Self::DisplayTimeFormat => b"settings/v1/display.time_format",
            Self::DisplayDateFormat => b"settings/v1/display.date_format",
            Self::DisplayUnits => b"settings/v1/display.units",
            Self::DisplayFirstDay => b"settings/v1/display.first_day",
            Self::DisplayTheme => b"settings/v1/display.theme",
            Self::DisplayScreenTimeout => b"settings/v1/display.screen_timeout",
            Self::DisplayAlwaysOn => b"settings/v1/display.always_on",
            Self::SoundVolume => b"settings/v1/sound.volume",
            Self::SoundMuted => b"settings/v1/sound.muted",
            Self::SoundDnd => b"settings/v1/sound.dnd",
            Self::SoundHaptics => b"settings/v1/sound.haptics",
            Self::SoundWakeSensitivity => b"settings/v1/sound.wake_sensitivity",
            Self::ConnectivityWifi => b"settings/v1/connectivity.wifi",
            Self::ConnectivityBluetooth => b"settings/v1/connectivity.bluetooth",
            Self::ConnectivitySync => b"settings/v1/connectivity.sync",
            Self::ConnectivityAutoSync => b"settings/v1/connectivity.auto_sync",
            Self::PowerWakeOnMotion => b"settings/v1/power.wake_on_motion",
            Self::PowerMotionSensitivity => b"settings/v1/power.motion_sensitivity",
            Self::PowerSleepTimeout => b"settings/v1/power.sleep_timeout",
            Self::PowerAwakeCap => b"settings/v1/power.awake_cap",
            Self::PowerPowerSaving => b"settings/v1/power.power_saving",
            Self::SystemTimezoneOffset => b"settings/v1/system.timezone_offset",
            Self::SystemDeviceName => b"settings/v1/system.device_name",
        }
    }
}
