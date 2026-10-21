//! Home complication values, formatted once per change.

use common::{Cpu1State, Percent, SystemStats, Timestamp};
use settings::{DateFormat, TimeFormat};

use crate::format::{clock_text, date_text, kb_text, percent_text, speed_text};

/// The data Home displays in both modes.
pub struct HomeState {
    last_minute: u64,
    battery: Percent,
    is_charging: bool,
    alarm: Option<Timestamp>,
    time_format: TimeFormat,
    date_format: DateFormat,
    stats: SystemStats,
    clock: [u8; 5],
    date: [u8; 10],
    battery_text: [u8; 4],
    battery_len: usize,
    cpu_buf: [u8; 4],
    cpu_len: usize,
    cpu1_buf: [u8; 4],
    cpu1_len: usize,
    ram_buf: [u8; 28],
    ram_len: usize,
    psram_buf: [u8; 28],
    psram_len: usize,
    speed_buf: [u8; 8],
    speed_len: usize,
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
        Self::new_with_formats(now, battery, is_charging, time_format, DateFormat::DayMonth)
    }

    /// Creates Home state with explicit clock and date formats.
    pub fn new_with_formats(
        now: Timestamp,
        battery: Percent,
        is_charging: bool,
        time_format: TimeFormat,
        date_format: DateFormat,
    ) -> Self {
        let mut state = Self {
            last_minute: u64::MAX,
            battery,
            is_charging,
            alarm: None,
            time_format,
            date_format,
            stats: SystemStats::empty(),
            clock: [0; 5],
            date: [0; 10],
            battery_text: [0; 4],
            battery_len: 0,
            cpu_buf: [0; 4],
            cpu_len: 0,
            cpu1_buf: [0; 4],
            cpu1_len: 0,
            ram_buf: [0; 28],
            ram_len: 0,
            psram_buf: [0; 28],
            psram_len: 0,
            speed_buf: [0; 8],
            speed_len: 0,
        };
        state.update(now, battery, is_charging);
        state.battery_len = percent_text(battery, &mut state.battery_text).len();
        state.write_stats();
        state
    }

    /// Updates the values and reports whether anything visible changed.
    pub fn update(&mut self, now: Timestamp, battery: Percent, is_charging: bool) -> bool {
        let mut changed = false;
        let minute = now.as_secs() / 60;
        if minute != self.last_minute {
            clock_text(now, self.time_format, &mut self.clock);
            date_text(now, self.date_format, &mut self.date);
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

    /// Updates the desk system stats, returning whether the sample changed.
    pub fn set_stats(&mut self, stats: SystemStats) -> bool {
        if stats == self.stats {
            return false;
        }
        self.stats = stats;
        self.write_stats();
        true
    }

    fn write_stats(&mut self) {
        let cpu = percent_text(self.stats.cpu_percent, &mut self.cpu_buf);
        self.cpu_len = cpu.len();
        self.cpu1_len = match self.stats.cpu1_state {
            Cpu1State::Off => {
                self.cpu1_buf[..3].copy_from_slice(b"off");
                3
            }
            Cpu1State::Error => {
                self.cpu1_buf[..3].copy_from_slice(b"err");
                3
            }
            _ => {
                let cpu1 = percent_text(self.stats.cpu1_percent, &mut self.cpu1_buf);
                cpu1.len()
            }
        };
        let ram = kb_text(
            self.stats.ram_used_kb,
            self.stats.ram_total_kb,
            &mut self.ram_buf,
        );
        self.ram_len = ram.len();
        let psram = kb_text(
            self.stats.psram_used_kb,
            self.stats.psram_total_kb,
            &mut self.psram_buf,
        );
        self.psram_len = psram.len();
        let speed = speed_text(self.stats.cpu_mhz, &mut self.speed_buf);
        self.speed_len = speed.len();
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

    /// Updates the date order, returning whether the text changed.
    pub fn set_date_format(&mut self, date_format: DateFormat) -> bool {
        if date_format == self.date_format {
            return false;
        }
        self.date_format = date_format;
        date_text(
            Timestamp::from_secs(self.last_minute.saturating_mul(60)),
            self.date_format,
            &mut self.date,
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

    /// The CPU0 load text.
    pub fn cpu(&self) -> &str {
        text(&self.cpu_buf[..self.cpu_len])
    }

    /// The CPU1 load or state text.
    pub fn cpu1(&self) -> &str {
        text(&self.cpu1_buf[..self.cpu1_len])
    }

    /// The internal SRAM text.
    pub fn ram(&self) -> &str {
        text(&self.ram_buf[..self.ram_len])
    }

    /// The PSRAM text.
    pub fn psram(&self) -> &str {
        text(&self.psram_buf[..self.psram_len])
    }

    /// The CPU clock text.
    pub fn speed(&self) -> &str {
        text(&self.speed_buf[..self.speed_len])
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
        assert_eq!(home.battery(), "80%");
        assert!(home.update(timestamp(7, 5), Percent::new(79), false));
        assert_eq!(home.battery(), "79%");
        assert!(home.update(timestamp(7, 5), Percent::new(79), true));
        assert!(home.is_charging());
    }

    #[test]
    fn stats_are_formatted_once_per_change() {
        let mut home = HomeState::new(timestamp(7, 5), Percent::new(80), false);
        let stats = SystemStats {
            cpu_percent: Percent::new(7),
            cpu1_percent: Percent::new(11),
            cpu1_state: Cpu1State::Audio,
            cpu_mhz: 240,
            ram_used_kb: 143,
            ram_total_kb: 320,
            psram_used_kb: 8192,
            psram_total_kb: 8192,
        };
        assert_eq!(home.cpu(), "00%");
        assert_eq!(home.cpu1(), "off");
        assert!(home.set_stats(stats));
        assert_eq!(home.cpu(), "07%");
        assert_eq!(home.cpu1(), "11%");
        assert_eq!(home.ram(), "143 / 320 KB");
        assert_eq!(home.psram(), "8192 / 8192 KB");
        assert_eq!(home.speed(), "240 MHz");
        assert!(!home.set_stats(stats));
        assert!(home.set_stats(SystemStats {
            cpu_percent: Percent::new(8),
            ..stats
        }));
        assert_eq!(home.cpu(), "08%");
        assert!(home.set_stats(SystemStats {
            cpu1_percent: Percent::new(0),
            cpu1_state: Cpu1State::Idle,
            ..stats
        }));
        assert_eq!(home.cpu1(), "00%");
        assert!(home.set_stats(SystemStats {
            cpu1_state: Cpu1State::Error,
            ..stats
        }));
        assert_eq!(home.cpu1(), "err");
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;
    use common::DateTime;
    use settings::DateFormat;

    #[test]
    fn date_order_updates_live() {
        let now = DateTime::new(2026, 10, 18, 7, 5, 0).to_timestamp().unwrap();
        let mut state = HomeState::new_with_formats(
            now,
            Percent::new(80),
            false,
            TimeFormat::Hour24,
            DateFormat::DayMonth,
        );
        assert_eq!(state.date(), "SUN 18 OCT");
        assert!(state.set_date_format(DateFormat::MonthDay));
        assert_eq!(state.date(), "SUN OCT 18");
        assert!(!state.set_date_format(DateFormat::MonthDay));
    }
}
