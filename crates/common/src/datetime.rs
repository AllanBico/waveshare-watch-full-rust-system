//! Gregorian calendar date/time and conversion to/from Unix time.

use crate::time::Timestamp;

/// Seconds in a day.
const SECS_PER_DAY: i64 = 86_400;

/// Lower bound of the RTC's two-digit (2000-2099) year window.
const MIN_YEAR: u16 = 2000;
/// Upper bound of the RTC's two-digit year window.
const MAX_YEAR: u16 = 2099;

/// A Gregorian calendar date and time in UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DateTime {
    /// Full year, e.g. 2026.
    pub year: u16,
    /// Month, 1-12.
    pub month: u8,
    /// Day of month, 1-31.
    pub day: u8,
    /// Hour, 0-23.
    pub hour: u8,
    /// Minute, 0-59.
    pub minute: u8,
    /// Second, 0-59.
    pub second: u8,
}

impl DateTime {
    /// Creates a date/time from its fields.
    pub const fn new(year: u16, month: u8, day: u8, hour: u8, minute: u8, second: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    /// Whether the fields form a real calendar date/time.
    pub const fn is_valid(&self) -> bool {
        self.month >= 1
            && self.month <= 12
            && self.day >= 1
            && self.day <= days_in_month(self.year, self.month)
            && self.hour < 24
            && self.minute < 60
            && self.second < 60
    }

    /// Day of week, 0 = Sunday .. 6 = Saturday, derived from the date.
    pub fn weekday(&self) -> u8 {
        let days = days_from_civil(self.year as i64, self.month as u32, self.day as u32);
        (((days % 7) + 7 + 4) % 7) as u8
    }

    /// Converts to a Unix timestamp, or `None` if the fields are invalid or
    /// before the epoch.
    pub fn to_timestamp(self) -> Option<Timestamp> {
        if !self.is_valid() {
            return None;
        }
        let days = days_from_civil(self.year as i64, self.month as u32, self.day as u32);
        let secs = days * SECS_PER_DAY
            + self.hour as i64 * 3600
            + self.minute as i64 * 60
            + self.second as i64;
        if secs < 0 {
            None
        } else {
            Some(Timestamp::from_secs(secs as u64))
        }
    }

    /// Builds a date/time from a Unix timestamp (UTC).
    pub fn from_timestamp(timestamp: Timestamp) -> Self {
        let secs = timestamp.as_secs() as i64;
        let days = secs / SECS_PER_DAY;
        let rem = secs % SECS_PER_DAY;
        let (year, month, day) = civil_from_days(days);
        Self {
            year: year as u16,
            month: month as u8,
            day: day as u8,
            hour: (rem / 3600) as u8,
            minute: ((rem % 3600) / 60) as u8,
            second: (rem % 60) as u8,
        }
    }
}

/// Days in `month` of `year`, accounting for leap years.
const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Whether `year` is a leap year in the Gregorian calendar.
const fn is_leap(year: u16) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp as i64 + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (y + i64::from(month <= 2), month, day)
}

/// Whether a year is within the RTC's supported range.
pub const fn is_supported_year(year: u16) -> bool {
    year >= MIN_YEAR && year <= MAX_YEAR
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970() {
        let dt = DateTime::from_timestamp(Timestamp::from_secs(0));
        assert_eq!(dt, DateTime::new(1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn known_timestamp_roundtrips() {
        let dt = DateTime::new(2026, 10, 17, 12, 34, 56);
        let ts = dt.to_timestamp().unwrap();
        assert_eq!(DateTime::from_timestamp(ts), dt);
    }

    #[test]
    fn leap_day_is_valid() {
        assert!(DateTime::new(2024, 2, 29, 0, 0, 0).is_valid());
        assert!(!DateTime::new(2023, 2, 29, 0, 0, 0).is_valid());
    }

    #[test]
    fn weekday_matches_known_days() {
        // 1970-01-01 was a Thursday (4); 2000-01-01 was a Saturday (6).
        assert_eq!(DateTime::new(1970, 1, 1, 0, 0, 0).weekday(), 4);
        assert_eq!(DateTime::new(2000, 1, 1, 0, 0, 0).weekday(), 6);
    }

    #[test]
    fn invalid_fields_reject() {
        assert!(!DateTime::new(2026, 13, 1, 0, 0, 0).is_valid());
        assert_eq!(DateTime::new(2026, 0, 1, 0, 0, 0).to_timestamp(), None);
    }
}
