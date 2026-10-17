//! Binary encoding for log entries.
//!
//! The schema version is part of every record. Unknown event tags, wake
//! reasons, or schema versions are rejected instead of guessed.

use core::fmt;

use crate::event::{Event, WakeReason};
use crate::log_entry::LogEntry;
use crate::mode::Mode;
use crate::time::Timestamp;
use crate::units::Millivolts;

const EVENT_WAKE: u8 = 0;
const EVENT_MODE: u8 = 1;
const EVENT_BATTERY: u8 = 2;
const EVENT_ERROR: u8 = 3;
const WAKE_TIMER: u8 = 0;
const WAKE_BUTTON: u8 = 1;
const WAKE_MOTION: u8 = 2;
const WAKE_USB: u8 = 3;
const WAKE_UNKNOWN: u8 = 4;
const MODE_WATCH: u8 = 0;
const MODE_DESK: u8 = 1;

/// Largest encoded record: schema + timestamp + tag + two payload bytes.
pub const MAX_ENCODED_LOG_LEN: usize = 1 + 8 + 1 + 2;

/// Why a log record could not be encoded or decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum LogError {
    /// The output buffer is too small.
    BufferTooSmall,
    /// The bytes are not a valid record.
    Corrupt,
    /// The record was written with a different schema version.
    VersionMismatch,
}

impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LogError::BufferTooSmall => "log buffer too small",
            LogError::Corrupt => "log record corrupt",
            LogError::VersionMismatch => "log schema mismatch",
        })
    }
}

/// Encodes a log entry into `out`, returning the used length.
pub fn encode_log(entry: &LogEntry, out: &mut [u8]) -> Result<usize, LogError> {
    if entry.schema_version != crate::SCHEMA_VERSION {
        return Err(LogError::VersionMismatch);
    }

    let mut index = 0;
    put_u8(out, &mut index, entry.schema_version)?;
    put_u64(out, &mut index, entry.timestamp.as_millis())?;
    match entry.event {
        Event::Wake(reason) => {
            put_u8(out, &mut index, EVENT_WAKE)?;
            put_u8(out, &mut index, wake_to_byte(reason)?)?;
        }
        Event::ModeSwitch { from, to } => {
            put_u8(out, &mut index, EVENT_MODE)?;
            put_u8(out, &mut index, mode_to_byte(from)?)?;
            put_u8(out, &mut index, mode_to_byte(to)?)?;
        }
        Event::Battery(millivolts) => {
            put_u8(out, &mut index, EVENT_BATTERY)?;
            put_u16(out, &mut index, millivolts.value())?;
        }
        Event::Error(code) => {
            put_u8(out, &mut index, EVENT_ERROR)?;
            put_u16(out, &mut index, code)?;
        }
    }
    Ok(index)
}

/// Decodes a log entry, rejecting trailing bytes.
pub fn decode_log(bytes: &[u8]) -> Result<LogEntry, LogError> {
    let mut index = 0;
    let schema_version = take_u8(bytes, &mut index)?;
    if schema_version != crate::SCHEMA_VERSION {
        return Err(LogError::VersionMismatch);
    }
    let timestamp = Timestamp::from_millis(take_u64(bytes, &mut index)?);
    let event = match take_u8(bytes, &mut index)? {
        EVENT_WAKE => Event::Wake(byte_to_wake(take_u8(bytes, &mut index)?)?),
        EVENT_MODE => Event::ModeSwitch {
            from: byte_to_mode(take_u8(bytes, &mut index)?)?,
            to: byte_to_mode(take_u8(bytes, &mut index)?)?,
        },
        EVENT_BATTERY => Event::Battery(Millivolts::new(take_u16(bytes, &mut index)?)),
        EVENT_ERROR => Event::Error(take_u16(bytes, &mut index)?),
        _ => return Err(LogError::Corrupt),
    };
    if index != bytes.len() {
        return Err(LogError::Corrupt);
    }
    Ok(LogEntry {
        schema_version,
        timestamp,
        event,
    })
}

fn wake_to_byte(reason: WakeReason) -> Result<u8, LogError> {
    match reason {
        WakeReason::Timer => Ok(WAKE_TIMER),
        WakeReason::Button => Ok(WAKE_BUTTON),
        WakeReason::Motion => Ok(WAKE_MOTION),
        WakeReason::Usb => Ok(WAKE_USB),
        WakeReason::Unknown => Ok(WAKE_UNKNOWN),
    }
}

fn byte_to_wake(value: u8) -> Result<WakeReason, LogError> {
    match value {
        WAKE_TIMER => Ok(WakeReason::Timer),
        WAKE_BUTTON => Ok(WakeReason::Button),
        WAKE_MOTION => Ok(WakeReason::Motion),
        WAKE_USB => Ok(WakeReason::Usb),
        WAKE_UNKNOWN => Ok(WakeReason::Unknown),
        _ => Err(LogError::Corrupt),
    }
}

fn mode_to_byte(mode: Mode) -> Result<u8, LogError> {
    match mode {
        Mode::Watch => Ok(MODE_WATCH),
        Mode::Desk => Ok(MODE_DESK),
    }
}

fn byte_to_mode(value: u8) -> Result<Mode, LogError> {
    match value {
        MODE_WATCH => Ok(Mode::Watch),
        MODE_DESK => Ok(Mode::Desk),
        _ => Err(LogError::Corrupt),
    }
}

fn put_u8(out: &mut [u8], index: &mut usize, value: u8) -> Result<(), LogError> {
    let slot = out.get_mut(*index).ok_or(LogError::BufferTooSmall)?;
    *slot = value;
    *index += 1;
    Ok(())
}

fn put_u16(out: &mut [u8], index: &mut usize, value: u16) -> Result<(), LogError> {
    for byte in value.to_le_bytes() {
        put_u8(out, index, byte)?;
    }
    Ok(())
}

fn put_u64(out: &mut [u8], index: &mut usize, value: u64) -> Result<(), LogError> {
    for byte in value.to_le_bytes() {
        put_u8(out, index, byte)?;
    }
    Ok(())
}

fn take_u8(bytes: &[u8], index: &mut usize) -> Result<u8, LogError> {
    let value = *bytes.get(*index).ok_or(LogError::Corrupt)?;
    *index += 1;
    Ok(value)
}

fn take_u16(bytes: &[u8], index: &mut usize) -> Result<u16, LogError> {
    let mut raw = [0u8; 2];
    for byte in &mut raw {
        *byte = take_u8(bytes, index)?;
    }
    Ok(u16::from_le_bytes(raw))
}

fn take_u64(bytes: &[u8], index: &mut usize) -> Result<u64, LogError> {
    let mut raw = [0u8; 8];
    for byte in &mut raw {
        *byte = take_u8(bytes, index)?;
    }
    Ok(u64::from_le_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wake_entry() -> LogEntry {
        LogEntry::new(
            Timestamp::from_millis(1_234),
            Event::Wake(WakeReason::Motion),
        )
    }

    fn switch_entry() -> LogEntry {
        LogEntry::new(
            Timestamp::from_secs(9),
            Event::ModeSwitch {
                from: Mode::Watch,
                to: Mode::Desk,
            },
        )
    }

    fn battery_entry() -> LogEntry {
        LogEntry::new(
            Timestamp::from_secs(10),
            Event::Battery(Millivolts::new(3_812)),
        )
    }

    fn error_entry() -> LogEntry {
        LogEntry::new(Timestamp::from_secs(11), Event::Error(3))
    }

    fn round_trip(entry: LogEntry, expected_len: usize) {
        let mut buffer = [0u8; MAX_ENCODED_LOG_LEN];
        let len = encode_log(&entry, &mut buffer).unwrap();
        assert_eq!(len, expected_len);
        assert_eq!(decode_log(&buffer[..len]).unwrap(), entry);
    }

    #[test]
    fn all_events_round_trip() {
        round_trip(wake_entry(), 11);
        round_trip(switch_entry(), 12);
        round_trip(battery_entry(), 12);
        round_trip(error_entry(), 12);
    }

    #[test]
    fn wrong_schema_is_rejected() {
        let mut entry = wake_entry();
        entry.schema_version = crate::SCHEMA_VERSION + 1;
        let mut buffer = [0u8; MAX_ENCODED_LOG_LEN];
        assert!(matches!(
            encode_log(&entry, &mut buffer),
            Err(LogError::VersionMismatch)
        ));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let entry = wake_entry();
        let mut buffer = [0u8; MAX_ENCODED_LOG_LEN];
        let len = encode_log(&entry, &mut buffer).unwrap();
        assert!(matches!(
            decode_log(&buffer[..len + 1]),
            Err(LogError::Corrupt)
        ));
    }

    #[test]
    fn unknown_tags_are_rejected() {
        let mut buffer = [0u8; MAX_ENCODED_LOG_LEN];
        let len = encode_log(&wake_entry(), &mut buffer).unwrap();
        buffer[9] = 0x7F;
        assert!(matches!(decode_log(&buffer[..len]), Err(LogError::Corrupt)));
    }
}
