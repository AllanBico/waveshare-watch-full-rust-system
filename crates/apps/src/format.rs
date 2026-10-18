//! Fixed-buffer text formatting for the Home complications.

use common::{DateTime, Percent, Timestamp};
use settings::TimeFormat;

const WEEKDAYS: [&str; 7] = ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"];
const MONTHS: [&str; 12] = [
    "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
];

/// Formats the clock into `out`.
pub(crate) fn clock_text(now: Timestamp, format: TimeFormat, out: &mut [u8; 5]) -> &str {
    let date = DateTime::from_timestamp(now);
    let hour = match format {
        TimeFormat::Hour24 => date.hour,
        TimeFormat::Hour12 => {
            let hour = date.hour % 12;
            if hour == 0 {
                12
            } else {
                hour
            }
        }
    };
    put_two(hour, out, 0);
    out[2] = b':';
    put_two(date.minute, out, 3);
    text(out)
}

/// Formats `DDD DD MMM` into `out`.
pub(crate) fn date_text(now: Timestamp, out: &mut [u8; 10]) -> &str {
    let date = DateTime::from_timestamp(now);
    let weekday = WEEKDAYS[usize::from(date.weekday()).min(WEEKDAYS.len() - 1)].as_bytes();
    let month = MONTHS[usize::from(date.month.clamp(1, 12) - 1)].as_bytes();
    out[..3].copy_from_slice(weekday);
    out[3] = b' ';
    put_two(date.day, out, 4);
    out[6] = b' ';
    out[7..10].copy_from_slice(month);
    text(out)
}

/// Formats a clamped percentage into `out`.
pub(crate) fn percent_text(percent: Percent, out: &mut [u8; 4]) -> &str {
    let value = percent.value();
    let length = if value == 100 {
        out[0] = b'1';
        out[1] = b'0';
        out[2] = b'0';
        3
    } else {
        put_two(value, out, 0);
        2
    };
    out[length] = b'%';
    text(&out[..length + 1])
}

fn put_two(value: u8, out: &mut [u8], index: usize) {
    let value = value.min(99);
    out[index] = b'0' + value / 10;
    out[index + 1] = b'0' + value % 10;
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(year: u16, month: u8, day: u8, hour: u8, minute: u8) -> Timestamp {
        DateTime::new(year, month, day, hour, minute, 0)
            .to_timestamp()
            .unwrap()
    }

    #[test]
    fn clock_is_zero_padded() {
        let mut buffer = [0; 5];
        assert_eq!(
            clock_text(at(2026, 10, 18, 7, 5), TimeFormat::Hour24, &mut buffer),
            "07:05"
        );
    }

    #[test]
    fn clock_uses_the_requested_format() {
        let mut buffer = [0; 5];
        assert_eq!(
            clock_text(at(2026, 10, 18, 19, 5), TimeFormat::Hour12, &mut buffer),
            "07:05"
        );
        assert_eq!(
            clock_text(at(2026, 10, 18, 0, 5), TimeFormat::Hour12, &mut buffer),
            "12:05"
        );
    }

    #[test]
    fn date_uses_short_names() {
        let mut buffer = [0; 10];
        assert_eq!(date_text(at(2026, 10, 18, 0, 0), &mut buffer), "SUN 18 OCT");
    }

    #[test]
    fn percent_handles_three_digits() {
        let mut buffer = [0; 4];
        assert_eq!(percent_text(Percent::new(7), &mut buffer), "07%");
        assert_eq!(percent_text(Percent::new(100), &mut buffer), "100%");
    }
}
