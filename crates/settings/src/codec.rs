//! Reading, validating, and writing settings by identifier.

use crate::value::SettingValue;
use crate::{Error, SettingId, Settings};

const MIN_SCREEN_TIMEOUT_SECS: u16 = 5;
const MAX_SCREEN_TIMEOUT_SECS: u16 = 300;
const MIN_SLEEP_TIMEOUT_SECS: u16 = 5;
const MAX_SLEEP_TIMEOUT_SECS: u16 = 3_600;
const MIN_AUTO_SYNC_SECS: u16 = 30;
const MAX_AUTO_SYNC_SECS: u16 = 65_535;
const MAX_TIMEZONE_OFFSET_MINUTES: i16 = 840;

impl SettingId {
    /// Decodes a stored value for this setting.
    pub fn decode(self, bytes: &[u8]) -> Result<SettingValue, Error> {
        let value = SettingValue::decode(self.kind(), bytes)?;
        if value.kind() != self.kind() {
            return Err(Error::WrongType);
        }
        Ok(value)
    }

    /// Applies a typed value, validating its range first.
    pub fn apply(self, settings: &mut Settings, value: SettingValue) -> Result<(), Error> {
        if value.kind() != self.kind() {
            return Err(Error::WrongType);
        }
        self.assign(settings, value)
    }

    /// Reads the current value for this setting.
    pub fn read(self, settings: &Settings) -> SettingValue {
        match self {
            Self::DisplayBrightnessWatch => {
                SettingValue::Percent(settings.display.brightness_watch)
            }
            Self::DisplayBrightnessDesk => SettingValue::Percent(settings.display.brightness_desk),
            Self::DisplayTimeFormat => SettingValue::TimeFormat(settings.display.time_format),
            Self::DisplayDateFormat => SettingValue::DateFormat(settings.display.date_format),
            Self::DisplayUnits => SettingValue::Units(settings.display.units),
            Self::DisplayFirstDay => SettingValue::FirstDay(settings.display.first_day),
            Self::DisplayTheme => SettingValue::Theme(settings.display.theme),
            Self::DisplayScreenTimeout => {
                SettingValue::Seconds(settings.display.screen_timeout_secs)
            }
            Self::DisplayAlwaysOn => SettingValue::Bool(settings.display.always_on),
            Self::SoundVolume => SettingValue::Percent(settings.sound.volume),
            Self::SoundMuted => SettingValue::Bool(settings.sound.is_muted),
            Self::SoundDnd => SettingValue::Bool(settings.sound.is_dnd),
            Self::SoundHaptics => SettingValue::Percent(settings.sound.haptics),
            Self::SoundWakeSensitivity => SettingValue::Percent(settings.sound.wake_sensitivity),
            Self::ConnectivityWifi => SettingValue::Bool(settings.connectivity.wifi),
            Self::ConnectivityBluetooth => SettingValue::Bool(settings.connectivity.bluetooth),
            Self::ConnectivitySync => SettingValue::Bool(settings.connectivity.sync),
            Self::ConnectivityAutoSync => {
                SettingValue::Seconds(settings.connectivity.auto_sync_secs)
            }
            Self::PowerWakeOnMotion => SettingValue::Bool(settings.power.wake_on_motion),
            Self::PowerMotionSensitivity => {
                SettingValue::Percent(settings.power.motion_sensitivity)
            }
            Self::PowerSleepTimeout => SettingValue::Seconds(settings.power.sleep_timeout_secs),
            Self::PowerPowerSaving => SettingValue::Bool(settings.power.power_saving),
            Self::SystemTimezoneOffset => {
                SettingValue::OffsetMinutes(settings.system.timezone_offset_minutes)
            }
            Self::SystemDeviceName => SettingValue::DeviceName(settings.system.device_name),
        }
    }

    fn assign(self, settings: &mut Settings, value: SettingValue) -> Result<(), Error> {
        match (self, value) {
            (Self::DisplayBrightnessWatch, SettingValue::Percent(value)) => {
                settings.display.brightness_watch = value
            }
            (Self::DisplayBrightnessDesk, SettingValue::Percent(value)) => {
                settings.display.brightness_desk = value
            }
            (Self::DisplayTimeFormat, SettingValue::TimeFormat(value)) => {
                settings.display.time_format = value
            }
            (Self::DisplayDateFormat, SettingValue::DateFormat(value)) => {
                settings.display.date_format = value
            }
            (Self::DisplayUnits, SettingValue::Units(value)) => settings.display.units = value,
            (Self::DisplayFirstDay, SettingValue::FirstDay(value)) => {
                settings.display.first_day = value
            }
            (Self::DisplayTheme, SettingValue::Theme(value)) => settings.display.theme = value,
            (Self::DisplayScreenTimeout, SettingValue::Seconds(value)) => {
                settings.display.screen_timeout_secs =
                    clamp_seconds(value, MIN_SCREEN_TIMEOUT_SECS, MAX_SCREEN_TIMEOUT_SECS)?
            }
            (Self::DisplayAlwaysOn, SettingValue::Bool(value)) => {
                settings.display.always_on = value
            }
            (Self::SoundVolume, SettingValue::Percent(value)) => settings.sound.volume = value,
            (Self::SoundMuted, SettingValue::Bool(value)) => settings.sound.is_muted = value,
            (Self::SoundDnd, SettingValue::Bool(value)) => settings.sound.is_dnd = value,
            (Self::SoundHaptics, SettingValue::Percent(value)) => settings.sound.haptics = value,
            (Self::SoundWakeSensitivity, SettingValue::Percent(value)) => {
                settings.sound.wake_sensitivity = value
            }
            (Self::ConnectivityWifi, SettingValue::Bool(value)) => {
                settings.connectivity.wifi = value
            }
            (Self::ConnectivityBluetooth, SettingValue::Bool(value)) => {
                settings.connectivity.bluetooth = value
            }
            (Self::ConnectivitySync, SettingValue::Bool(value)) => {
                settings.connectivity.sync = value
            }
            (Self::ConnectivityAutoSync, SettingValue::Seconds(value)) => {
                settings.connectivity.auto_sync_secs =
                    clamp_seconds(value, MIN_AUTO_SYNC_SECS, MAX_AUTO_SYNC_SECS)?
            }
            (Self::PowerWakeOnMotion, SettingValue::Bool(value)) => {
                settings.power.wake_on_motion = value
            }
            (Self::PowerMotionSensitivity, SettingValue::Percent(value)) => {
                settings.power.motion_sensitivity = value
            }
            (Self::PowerSleepTimeout, SettingValue::Seconds(value)) => {
                settings.power.sleep_timeout_secs =
                    clamp_seconds(value, MIN_SLEEP_TIMEOUT_SECS, MAX_SLEEP_TIMEOUT_SECS)?
            }
            (Self::PowerPowerSaving, SettingValue::Bool(value)) => {
                settings.power.power_saving = value
            }
            (Self::SystemTimezoneOffset, SettingValue::OffsetMinutes(value)) => {
                settings.system.timezone_offset_minutes =
                    value.clamp(-MAX_TIMEZONE_OFFSET_MINUTES, MAX_TIMEZONE_OFFSET_MINUTES)
            }
            (Self::SystemDeviceName, SettingValue::DeviceName(value)) => {
                settings.system.device_name = value
            }
            _ => return Err(Error::WrongType),
        }
        Ok(())
    }
}

fn clamp_seconds(value: u16, min: u16, max: u16) -> Result<u16, Error> {
    if value < min || value > max {
        return Err(Error::OutOfRange);
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::TimeFormat;
    use common::Percent;

    #[test]
    fn every_default_roundtrips() {
        let mut buffer = [0u8; crate::value::MAX_ENCODED_VALUE_LEN];
        for id in SettingId::ALL {
            let value = id.default_value();
            let len = value.encode(&mut buffer).unwrap();
            assert_eq!(id.decode(&buffer[..len]), Ok(value), "{:?}", id);
        }
    }

    #[test]
    fn out_of_range_durations_are_rejected() {
        let mut settings = Settings::new();
        assert_eq!(
            SettingId::DisplayScreenTimeout.apply(&mut settings, SettingValue::Seconds(1)),
            Err(Error::OutOfRange)
        );
        assert_eq!(
            SettingId::ConnectivityAutoSync.apply(&mut settings, SettingValue::Seconds(1)),
            Err(Error::OutOfRange)
        );
    }

    #[test]
    fn timezone_is_clamped() {
        let mut settings = Settings::new();
        let id = SettingId::SystemTimezoneOffset;
        assert_eq!(
            id.apply(&mut settings, SettingValue::OffsetMinutes(9_000)),
            Ok(())
        );
        assert_eq!(settings.system.timezone_offset_minutes, 840);
    }

    #[test]
    fn wrong_type_is_rejected() {
        let mut settings = Settings::new();
        assert_eq!(
            SettingId::DisplayTheme.apply(&mut settings, SettingValue::Bool(true)),
            Err(Error::WrongType)
        );
    }

    #[test]
    fn read_returns_the_assigned_value() {
        let mut settings = Settings::new();
        let id = SettingId::DisplayTimeFormat;
        id.apply(&mut settings, SettingValue::TimeFormat(TimeFormat::Hour12))
            .unwrap();
        assert_eq!(
            id.read(&settings),
            SettingValue::TimeFormat(TimeFormat::Hour12)
        );
    }

    #[test]
    fn percent_values_are_validated_by_type() {
        let value = SettingValue::Percent(Percent::new(101));
        assert_eq!(value, SettingValue::Percent(Percent::new(100)));
    }
}
