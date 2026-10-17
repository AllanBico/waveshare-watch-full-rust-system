//! Binary encoding for log entries on flash.
//!
//! The schema version is part of every record. Unknown event tags, wake
//! reasons, or schema versions are rejected instead of guessed.

use common::{Event, LogEntry, Mode, Timestamp, WakeReason};

use super::Error;

const EVENT_WAKE: u8 = 0;
const EVENT_MODE: u8 = 1;
const WAKE_TIMER: u8 = 0;
const WAKE_BUTTON: u8 = 1;
const WAKE_MOTION: u8 = 2;
const WAKE_USB: u8 = 3;
const WAKE_UNKNOWN: u8 = 4;
const MODE_WATCH: u8 = 0;
const MODE_DESK: u8 = 1;

/// Largest encoded record: schema + timestamp + tag + two payload bytes.
pub const MAX_ENCODED_LEN: usize = 1 + 8 + 1 + 2;

/// Encodes a log entry into `out`, returning the used length.
pub fn encode(entry: &LogEntry, out: &mut [u8]) -> Result<usize, Error> {
    if entry.schema_version != common::SCHEMA_VERSION {
        return Err(Error::VersionMismatch);
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
        _ => return Err(Error::Corrupt),
    }
    Ok(index)
}

/// Decodes a log entry, rejecting trailing bytes.
pub fn decode(bytes: &[u8]) -> Result<LogEntry, Error> {
    let mut index = 0;
    let schema_version = take_u8(bytes, &mut index)?;
    if schema_version != common::SCHEMA_VERSION {
        return Err(Error::VersionMismatch);
    }
    let timestamp = Timestamp::from_millis(take_u64(bytes, &mut index)?);
    let event = match take_u8(bytes, &mut index)? {
        EVENT_WAKE => Event::Wake(byte_to_wake(take_u8(bytes, &mut index)?)?),
        EVENT_MODE => Event::ModeSwitch {
            from: byte_to_mode(take_u8(bytes, &mut index)?)?,
            to: byte_to_mode(take_u8(bytes, &mut index)?)?,
        },
        _ => return Err(Error::Corrupt),
    };
    if index != bytes.len() {
        return Err(Error::Corrupt);
    }
    Ok(LogEntry {
        schema_version,
        timestamp,
        event,
    })
}

fn wake_to_byte(reason: WakeReason) -> Result<u8, Error> {
    match reason {
        WakeReason::Timer => Ok(WAKE_TIMER),
        WakeReason::Button => Ok(WAKE_BUTTON),
        WakeReason::Motion => Ok(WAKE_MOTION),
        WakeReason::Usb => Ok(WAKE_USB),
        WakeReason::Unknown => Ok(WAKE_UNKNOWN),
        _ => Err(Error::Corrupt),
    }
}

fn byte_to_wake(value: u8) -> Result<WakeReason, Error> {
    match value {
        WAKE_TIMER => Ok(WakeReason::Timer),
        WAKE_BUTTON => Ok(WakeReason::Button),
        WAKE_MOTION => Ok(WakeReason::Motion),
        WAKE_USB => Ok(WakeReason::Usb),
        WAKE_UNKNOWN => Ok(WakeReason::Unknown),
        _ => Err(Error::Corrupt),
    }
}

fn mode_to_byte(mode: Mode) -> Result<u8, Error> {
    match mode {
        Mode::Watch => Ok(MODE_WATCH),
        Mode::Desk => Ok(MODE_DESK),
    }
}

fn byte_to_mode(value: u8) -> Result<Mode, Error> {
    match value {
        MODE_WATCH => Ok(Mode::Watch),
        MODE_DESK => Ok(Mode::Desk),
        _ => Err(Error::Corrupt),
    }
}

fn put_u8(out: &mut [u8], index: &mut usize, value: u8) -> Result<(), Error> {
    let slot = out.get_mut(*index).ok_or(Error::Corrupt)?;
    *slot = value;
    *index += 1;
    Ok(())
}

fn put_u64(out: &mut [u8], index: &mut usize, value: u64) -> Result<(), Error> {
    for byte in value.to_le_bytes() {
        put_u8(out, index, byte)?;
    }
    Ok(())
}

fn take_u8(bytes: &[u8], index: &mut usize) -> Result<u8, Error> {
    let value = *bytes.get(*index).ok_or(Error::Corrupt)?;
    *index += 1;
    Ok(value)
}

fn take_u64(bytes: &[u8], index: &mut usize) -> Result<u64, Error> {
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

    #[test]
    fn wake_round_trips() {
        let entry = wake_entry();
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let len = encode(&entry, &mut buffer).unwrap();
        assert_eq!(len, 11);
        assert_eq!(decode(&buffer[..len]).unwrap(), entry);
    }

    #[test]
    fn mode_switch_round_trips() {
        let entry = switch_entry();
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let len = encode(&entry, &mut buffer).unwrap();
        assert_eq!(len, 12);
        assert_eq!(decode(&buffer[..len]).unwrap(), entry);
    }

    #[test]
    fn wrong_schema_is_rejected() {
        let mut entry = wake_entry();
        entry.schema_version = common::SCHEMA_VERSION + 1;
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        assert!(matches!(
            encode(&entry, &mut buffer),
            Err(Error::VersionMismatch)
        ));
    }

    #[test]
    fn trailing_bytes_are_rejected() {
        let entry = wake_entry();
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let len = encode(&entry, &mut buffer).unwrap();
        assert!(matches!(decode(&buffer[..len + 1]), Err(Error::Corrupt)));
    }

    #[test]
    fn unknown_tags_are_rejected() {
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let len = encode(&wake_entry(), &mut buffer).unwrap();
        buffer[9] = 0x7F;
        assert!(matches!(decode(&buffer[..len]), Err(Error::Corrupt)));
    }
}
