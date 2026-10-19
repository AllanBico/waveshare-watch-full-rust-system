//! Setting value types and encoding.

use common::{Percent, ThemeId};

use crate::Error;

/// Longest device name accepted by a setting.
pub const MAX_DEVICE_NAME_LEN: usize = 24;
/// Largest encoded setting value.
pub const MAX_ENCODED_VALUE_LEN: usize = MAX_DEVICE_NAME_LEN + 1;

/// Clock display format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum TimeFormat {
    /// 24-hour clock.
    Hour24 = 0,
    /// 12-hour clock.
    Hour12 = 1,
}

impl TimeFormat {
    /// Toggles between the two formats.
    pub const fn next(self) -> Self {
        match self {
            Self::Hour24 => Self::Hour12,
            Self::Hour12 => Self::Hour24,
        }
    }

    /// Short UI label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Hour24 => "24-hour",
            Self::Hour12 => "12-hour",
        }
    }

    pub(crate) const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Hour24),
            1 => Some(Self::Hour12),
            _ => None,
        }
    }
}

/// Date display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum DateFormat {
    /// Day before month.
    DayMonth = 0,
    /// Month before day.
    MonthDay = 1,
}

impl DateFormat {
    /// Toggles between the two orders.
    pub const fn next(self) -> Self {
        match self {
            Self::DayMonth => Self::MonthDay,
            Self::MonthDay => Self::DayMonth,
        }
    }

    /// Short UI label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::DayMonth => "Day month",
            Self::MonthDay => "Month day",
        }
    }

    pub(crate) const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::DayMonth),
            1 => Some(Self::MonthDay),
            _ => None,
        }
    }
}

/// Measurement units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Units {
    /// Metric units.
    Metric = 0,
    /// Imperial units.
    Imperial = 1,
}

impl Units {
    pub(crate) const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Metric),
            1 => Some(Self::Imperial),
            _ => None,
        }
    }
}

/// First day of the week.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum FirstDay {
    /// Sunday.
    Sunday = 0,
    /// Monday.
    Monday = 1,
}

impl FirstDay {
    pub(crate) const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Sunday),
            1 => Some(Self::Monday),
            _ => None,
        }
    }
}

/// A fixed-capacity, printable device name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DeviceName {
    bytes: [u8; MAX_DEVICE_NAME_LEN],
    len: u8,
}

impl DeviceName {
    /// The default name, `wv2`.
    pub const fn new() -> Self {
        let mut bytes = [0u8; MAX_DEVICE_NAME_LEN];
        bytes[0] = b'w';
        bytes[1] = b'v';
        bytes[2] = b'2';
        Self { bytes, len: 3 }
    }

    /// Copies a printable ASCII name.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.is_empty() || bytes.len() > MAX_DEVICE_NAME_LEN {
            return Err(Error::OutOfRange);
        }
        if !bytes.iter().all(|byte| (0x20..=0x7e).contains(byte)) {
            return Err(Error::OutOfRange);
        }
        let mut name = Self::new();
        name.bytes = [0; MAX_DEVICE_NAME_LEN];
        name.bytes[..bytes.len()].copy_from_slice(bytes);
        name.len = bytes.len() as u8;
        Ok(name)
    }

    /// The name as text.
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..usize::from(self.len)]).unwrap_or("")
    }

    /// Raw name bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..usize::from(self.len)]
    }

    /// Encodes the name as a length byte followed by the name bytes.
    pub fn encode(self, out: &mut [u8]) -> Result<usize, Error> {
        let len = usize::from(self.len);
        if out.len() < len + 1 {
            return Err(Error::BufferTooSmall);
        }
        out[0] = self.len;
        out[1..len + 1].copy_from_slice(self.as_bytes());
        Ok(len + 1)
    }

    /// Decodes a length-prefixed name.
    pub fn decode(bytes: &[u8]) -> Result<Self, Error> {
        let Some((&len, name)) = bytes.split_first() else {
            return Err(Error::InvalidEncoding);
        };
        let len = usize::from(len);
        if name.len() != len {
            return Err(Error::InvalidEncoding);
        }
        Self::from_bytes(name)
    }
}

impl Default for DeviceName {
    fn default() -> Self {
        Self::new()
    }
}

/// The type expected by a setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ValueKind {
    /// On or off.
    Bool,
    /// Percentage in `0..=100`.
    Percent,
    /// Seconds, validated per setting.
    Seconds,
    /// Signed minutes from UTC.
    OffsetMinutes,
    /// Clock format.
    TimeFormat,
    /// Date format.
    DateFormat,
    /// Measurement units.
    Units,
    /// First day of the week.
    FirstDay,
    /// Theme variant.
    Theme,
    /// Device name.
    DeviceName,
}

/// One typed setting value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SettingValue {
    /// On or off.
    Bool(bool),
    /// Percentage.
    Percent(Percent),
    /// Duration in seconds.
    Seconds(u16),
    /// Signed minutes from UTC.
    OffsetMinutes(i16),
    /// Clock format.
    TimeFormat(TimeFormat),
    /// Date format.
    DateFormat(DateFormat),
    /// Measurement units.
    Units(Units),
    /// First day of the week.
    FirstDay(FirstDay),
    /// Theme variant.
    Theme(ThemeId),
    /// Device name.
    DeviceName(DeviceName),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_name_roundtrips_and_rejects_bad_input() {
        let name = DeviceName::from_bytes(b"desk").unwrap();
        let mut buffer = [0u8; MAX_ENCODED_VALUE_LEN];
        let len = name.encode(&mut buffer).unwrap();
        assert_eq!(DeviceName::decode(&buffer[..len]), Ok(name));
        assert_eq!(name.as_str(), "desk");
        assert!(DeviceName::from_bytes(b"").is_err());
        assert!(DeviceName::from_bytes(b"\xff").is_err());
    }

    #[test]
    fn values_roundtrip() {
        let cases = [
            (SettingValue::Bool(true), ValueKind::Bool),
            (SettingValue::Percent(Percent::new(70)), ValueKind::Percent),
            (SettingValue::Seconds(300), ValueKind::Seconds),
            (SettingValue::OffsetMinutes(-60), ValueKind::OffsetMinutes),
            (
                SettingValue::TimeFormat(TimeFormat::Hour12),
                ValueKind::TimeFormat,
            ),
            (SettingValue::Theme(ThemeId::HighContrast), ValueKind::Theme),
            (
                SettingValue::DeviceName(DeviceName::from_bytes(b"wv2").unwrap()),
                ValueKind::DeviceName,
            ),
        ];
        let mut buffer = [0u8; MAX_ENCODED_VALUE_LEN];
        for (value, kind) in cases {
            let len = value.encode(&mut buffer).unwrap();
            assert_eq!(SettingValue::decode(kind, &buffer[..len]), Ok(value));
        }
    }

    #[test]
    fn encoded_values_reject_wrong_lengths() {
        assert_eq!(
            SettingValue::decode(ValueKind::Bool, &[]),
            Err(Error::InvalidEncoding)
        );
        assert_eq!(
            SettingValue::decode(ValueKind::Seconds, &[1]),
            Err(Error::InvalidEncoding)
        );
    }
}
