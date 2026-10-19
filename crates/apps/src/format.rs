//! Fixed-buffer text formatting for the Home complications.

use common::{DateTime, Percent, Timestamp};
use settings::{DateFormat, TimeFormat};

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

/// Formats `DDD DD MMM` or `DDD MMM DD` into `out`.
pub(crate) fn date_text(now: Timestamp, format: DateFormat, out: &mut [u8; 10]) -> &str {
    let date = DateTime::from_timestamp(now);
    let weekday = WEEKDAYS[usize::from(date.weekday()).min(WEEKDAYS.len() - 1)].as_bytes();
    let month = MONTHS[usize::from(date.month.clamp(1, 12) - 1)].as_bytes();
    out[..3].copy_from_slice(weekday);
    out[3] = b' ';
    match format {
        DateFormat::DayMonth => {
            put_two(date.day, out, 4);
            out[6] = b' ';
            out[7..10].copy_from_slice(month);
        }
        DateFormat::MonthDay => {
            out[4..7].copy_from_slice(month);
            out[7] = b' ';
            put_two(date.day, out, 8);
        }
    }
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

/// Formats `used / total KB` into `out`.
pub(crate) fn kb_text(used: u32, total: u32, out: &mut [u8; 28]) -> &str {
    let mut index = put_u32(used, out, 0);
    out[index..index + 3].copy_from_slice(b" / ");
    index += 3;
    index = put_u32(total, out, index);
    out[index..index + 3].copy_from_slice(b" KB");
    text(&out[..index + 3])
}

/// Formats a CPU clock in MHz into `out`.
pub(crate) fn speed_text(mhz: u16, out: &mut [u8; 8]) -> &str {
    let index = put_u32(u32::from(mhz), out, 0);
    out[index..index + 4].copy_from_slice(b" MHz");
    text(&out[..index + 4])
}

fn put_u32(value: u32, out: &mut [u8], mut index: usize) -> usize {
    let mut digits = [0u8; 10];
    let mut count = 0;
    let mut value = value;
    loop {
        digits[count] = b'0' + (value % 10) as u8;
        count += 1;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    while count > 0 {
        count -= 1;
        out[index] = digits[count];
        index += 1;
    }
    index
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
        assert_eq!(
            date_text(at(2026, 10, 18, 0, 0), DateFormat::DayMonth, &mut buffer),
            "SUN 18 OCT"
        );
        assert_eq!(
            date_text(at(2026, 10, 18, 0, 0), DateFormat::MonthDay, &mut buffer),
            "SUN OCT 18"
        );
    }

    #[test]
    fn percent_handles_three_digits() {
        let mut buffer = [0; 4];
        assert_eq!(percent_text(Percent::new(7), &mut buffer), "07%");
        assert_eq!(percent_text(Percent::new(100), &mut buffer), "100%");
    }

    #[test]
    fn kb_pairs_are_plain_decimals() {
        let mut buffer = [0; 28];
        assert_eq!(kb_text(0, 0, &mut buffer), "0 / 0 KB");
        assert_eq!(kb_text(143, 320, &mut buffer), "143 / 320 KB");
        assert_eq!(kb_text(8192, 8192, &mut buffer), "8192 / 8192 KB");
    }

    #[test]
    fn speed_keeps_the_unit() {
        let mut buffer = [0; 8];
        assert_eq!(speed_text(0, &mut buffer), "0 MHz");
        assert_eq!(speed_text(80, &mut buffer), "80 MHz");
        assert_eq!(speed_text(240, &mut buffer), "240 MHz");
    }
}
