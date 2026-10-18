//! Encoding and decoding of setting values.

use common::{Percent, ThemeId};

use super::value::{DateFormat, DeviceName, FirstDay, SettingValue, TimeFormat, Units, ValueKind};
use crate::Error;

impl SettingValue {
    /// The value's kind.
    pub const fn kind(self) -> ValueKind {
        match self {
            Self::Bool(_) => ValueKind::Bool,
            Self::Percent(_) => ValueKind::Percent,
            Self::Seconds(_) => ValueKind::Seconds,
            Self::OffsetMinutes(_) => ValueKind::OffsetMinutes,
            Self::TimeFormat(_) => ValueKind::TimeFormat,
            Self::DateFormat(_) => ValueKind::DateFormat,
            Self::Units(_) => ValueKind::Units,
            Self::FirstDay(_) => ValueKind::FirstDay,
            Self::Theme(_) => ValueKind::Theme,
            Self::DeviceName(_) => ValueKind::DeviceName,
        }
    }

    /// Encodes the value into `out`, returning the encoded length.
    pub fn encode(self, out: &mut [u8]) -> Result<usize, Error> {
        match self {
            Self::Bool(value) => write_byte(u8::from(value), out),
            Self::Percent(value) => write_byte(value.value(), out),
            Self::Seconds(value) => write_bytes(&value.to_le_bytes(), out),
            Self::OffsetMinutes(value) => write_bytes(&value.to_le_bytes(), out),
            Self::TimeFormat(value) => write_byte(value as u8, out),
            Self::DateFormat(value) => write_byte(value as u8, out),
            Self::Units(value) => write_byte(value as u8, out),
            Self::FirstDay(value) => write_byte(value as u8, out),
            Self::Theme(value) => write_byte(value.index(), out),
            Self::DeviceName(value) => value.encode(out),
        }
    }

    /// Decodes a value of the expected kind from `bytes`.
    pub fn decode(kind: ValueKind, bytes: &[u8]) -> Result<Self, Error> {
        match kind {
            ValueKind::Bool => match exactly_one(bytes)? {
                0 => Ok(Self::Bool(false)),
                1 => Ok(Self::Bool(true)),
                _ => Err(Error::InvalidEncoding),
            },
            ValueKind::Percent => match exactly_one(bytes)? {
                value if value <= 100 => Ok(Self::Percent(Percent::new(value))),
                _ => Err(Error::InvalidEncoding),
            },
            ValueKind::Seconds => Ok(Self::Seconds(u16::from_le_bytes(exactly_two(bytes)?))),
            ValueKind::OffsetMinutes => {
                Ok(Self::OffsetMinutes(i16::from_le_bytes(exactly_two(bytes)?)))
            }
            ValueKind::TimeFormat => TimeFormat::from_index(exactly_one(bytes)?)
                .map(Self::TimeFormat)
                .ok_or(Error::InvalidEncoding),
            ValueKind::DateFormat => DateFormat::from_index(exactly_one(bytes)?)
                .map(Self::DateFormat)
                .ok_or(Error::InvalidEncoding),
            ValueKind::Units => Units::from_index(exactly_one(bytes)?)
                .map(Self::Units)
                .ok_or(Error::InvalidEncoding),
            ValueKind::FirstDay => FirstDay::from_index(exactly_one(bytes)?)
                .map(Self::FirstDay)
                .ok_or(Error::InvalidEncoding),
            ValueKind::Theme => ThemeId::from_index(exactly_one(bytes)?)
                .map(Self::Theme)
                .ok_or(Error::InvalidEncoding),
            ValueKind::DeviceName => DeviceName::decode(bytes).map(Self::DeviceName),
        }
    }
}

fn write_byte(value: u8, out: &mut [u8]) -> Result<usize, Error> {
    let Some(slot) = out.first_mut() else {
        return Err(Error::BufferTooSmall);
    };
    *slot = value;
    Ok(1)
}

fn write_bytes(bytes: &[u8], out: &mut [u8]) -> Result<usize, Error> {
    let Some(target) = out.get_mut(..bytes.len()) else {
        return Err(Error::BufferTooSmall);
    };
    target.copy_from_slice(bytes);
    Ok(bytes.len())
}

fn exactly_one(bytes: &[u8]) -> Result<u8, Error> {
    match bytes {
        [value] => Ok(*value),
        _ => Err(Error::InvalidEncoding),
    }
}

fn exactly_two(bytes: &[u8]) -> Result<[u8; 2], Error> {
    match bytes {
        [a, b] => Ok([*a, *b]),
        _ => Err(Error::InvalidEncoding),
    }
}
