//! Time-series log record.

use crate::event::Event;
use crate::time::Timestamp;

/// On-disk log schema version.
///
/// Bump on any breaking change to [`LogEntry`] layout, so stored and synced
/// records stay self-describing.
pub const SCHEMA_VERSION: u8 = 1;

/// A single record in the time-series log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LogEntry {
    /// Schema version this record was written with.
    pub schema_version: u8,
    /// When the event happened.
    pub timestamp: Timestamp,
    /// What happened.
    pub event: Event,
}

impl LogEntry {
    /// Creates an entry stamped with the current [`SCHEMA_VERSION`].
    pub const fn new(timestamp: Timestamp, event: Event) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            timestamp,
            event,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::WakeReason;

    #[test]
    fn new_stamps_schema_version() {
        let entry = LogEntry::new(Timestamp::from_millis(1), Event::Wake(WakeReason::Timer));
        assert_eq!(entry.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn fields_are_accessible() {
        let entry = LogEntry::new(Timestamp::from_secs(5), Event::Wake(WakeReason::Button));
        assert_eq!(entry.timestamp.as_secs(), 5);
        assert_eq!(entry.event, Event::Wake(WakeReason::Button));
    }
}
