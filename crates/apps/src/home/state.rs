//! Home complication values, formatted once per change.

use common::{Percent, Timestamp};
use settings::TimeFormat;

use crate::format::{clock_text, date_text, percent_text};

/// The data Home displays in both modes.
pub struct HomeState {
    last_minute: u64,
    battery: Percent,
    is_charging: bool,
    alarm: Option<Timestamp>,
    time_format: TimeFormat,
    clock: [u8; 5],
    date: [u8; 10],
    battery_text: [u8; 4],
    battery_len: usize,
}

impl HomeState {
    /// Creates Home state with a 24-hour clock.
    pub fn new(now: Timestamp, battery: Percent, is_charging: bool) -> Self {
        Self::new_with_format(now, battery, is_charging, TimeFormat::Hour24)
    }

    /// Creates Home state with an explicit clock format.
    pub fn new_with_format(
        now: Timestamp,
        battery: Percent,
        is_charging: bool,
        time_format: TimeFormat,
    ) -> Self {
        let mut state = Self {
            last_minute: u64::MAX,
            battery,
            is_charging,
            alarm: None,
            time_format,
            clock: [0; 5],
            date: [0; 10],
            battery_text: [0; 4],
            battery_len: 0,
        };
        state.update(now, battery, is_charging);
        state
    }

    /// Updates the values and reports whether anything visible changed.
    pub fn update(&mut self, now: Timestamp, battery: Percent, is_charging: bool) -> bool {
        let mut changed = false;
        let minute = now.as_secs() / 60;
        if minute != self.last_minute {
            clock_text(now, self.time_format, &mut self.clock);
            date_text(now, &mut self.date);
            self.last_minute = minute;
            changed = true;
        }
        if battery != self.battery {
            let text = percent_text(battery, &mut self.battery_text);
            self.battery_len = text.len();
            self.battery = battery;
            changed = true;
        }
        if is_charging != self.is_charging {
            self.is_charging = is_charging;
            changed = true;
        }
        changed
    }

    /// Updates the clock format, returning whether the text changed.
    pub fn set_time_format(&mut self, time_format: TimeFormat) -> bool {
        if time_format == self.time_format {
            return false;
        }
        self.time_format = time_format;
        clock_text(
            Timestamp::from_secs(self.last_minute.saturating_mul(60)),
            self.time_format,
            &mut self.clock,
        );
        true
    }

    /// The clock text.
    pub fn clock(&self) -> &str {
        text(&self.clock)
    }

    /// The short date text.
    pub fn date(&self) -> &str {
        text(&self.date)
    }

    /// The battery percentage text.
    pub fn battery(&self) -> &str {
        text(&self.battery_text[..self.battery_len])
    }

    /// Whether the battery is charging.
    pub const fn is_charging(&self) -> bool {
        self.is_charging
    }

    /// The alarm ringing this wake, if any.
    pub const fn alarm(&self) -> Option<Timestamp> {
        self.alarm
    }

    /// Updates the ringing alarm, returning whether it changed.
    pub fn set_alarm(&mut self, alarm: Option<Timestamp>) -> bool {
        if self.alarm == alarm {
            return false;
        }
        self.alarm = alarm;
        true
    }
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::DateTime;

    fn timestamp(hour: u8, minute: u8) -> Timestamp {
        DateTime::new(2026, 10, 18, hour, minute, 0)
            .to_timestamp()
            .unwrap()
    }

    #[test]
    fn a_new_minute_updates_the_clock() {
        let mut home = HomeState::new(timestamp(7, 5), Percent::new(80), false);
        assert_eq!(home.clock(), "07:05");
        assert!(!home.update(timestamp(7, 5), Percent::new(80), false));
        assert!(home.update(timestamp(7, 6), Percent::new(80), false));
        assert_eq!(home.clock(), "07:06");
    }

    #[test]
    fn battery_and_charge_changes_are_reported() {
        let mut home = HomeState::new(timestamp(7, 5), Percent::new(80), false);
        assert!(home.update(timestamp(7, 5), Percent::new(79), false));
        assert_eq!(home.battery(), "79%");
        assert!(home.update(timestamp(7, 5), Percent::new(79), true));
        assert!(home.is_charging());
    }
}
