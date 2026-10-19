//! Wake and battery records emitted on every boot.

use common::{Event, LogEntry, Millivolts, Timestamp, WakeReason};

/// Builds the two records every wake must retain: its reason and battery.
pub const fn wake_entries(
    now: Timestamp,
    wake: Option<WakeReason>,
    battery: Millivolts,
) -> [LogEntry; 2] {
    let reason = match wake {
        Some(reason) => reason,
        None => WakeReason::Unknown,
    };
    [
        LogEntry::new(now, Event::Wake(reason)),
        LogEntry::new(now, Event::Battery(battery)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hundred_wake_cycles_record_reason_and_battery() {
        for cycle in 0..100u16 {
            let battery = Millivolts::new(3_700 + cycle);
            let entries = wake_entries(
                Timestamp::from_secs(u64::from(cycle)),
                Some(WakeReason::Timer),
                battery,
            );
            assert_eq!(entries[0].event, Event::Wake(WakeReason::Timer));
            assert_eq!(entries[1].event, Event::Battery(battery));
        }
    }

    #[test]
    fn a_cold_boot_records_an_unknown_reason() {
        let entries = wake_entries(Timestamp::from_secs(0), None, Millivolts::new(3_800));
        assert_eq!(entries[0].event, Event::Wake(WakeReason::Unknown));
    }
}
