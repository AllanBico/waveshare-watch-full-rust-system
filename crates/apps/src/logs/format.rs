//! Fixed-buffer formatting for log rows.

use common::{DateTime, Event, LogEntry, Millivolts, Mode, Timestamp, WakeReason};

const TEXT_LEN: usize = 28;

/// Formats a log timestamp as `MM-DD HH:MM`, or `--:--` when unset.
pub(super) fn time_text<'a>(timestamp: Timestamp, out: &'a mut [u8; 12]) -> &'a str {
    if timestamp.as_millis() == 0 {
        return "--:--";
    }
    let date = DateTime::from_timestamp(timestamp);
    let mut index = 0usize;
    put_two(out, &mut index, date.month);
    out[index] = b'-';
    index += 1;
    put_two(out, &mut index, date.day);
    out[index] = b' ';
    index += 1;
    put_two(out, &mut index, date.hour);
    out[index] = b':';
    index += 1;
    put_two(out, &mut index, date.minute);
    text(out, index)
}

/// Formats the event portion of a log entry.
pub(super) fn event_text<'a>(entry: &LogEntry, out: &'a mut [u8; TEXT_LEN]) -> &'a str {
    let mut index = 0usize;
    match entry.event {
        Event::Wake(reason) => wake_text(reason, out, &mut index),
        Event::Battery(millivolts) => battery_text(millivolts, out, &mut index),
        Event::ModeSwitch { from, to } => {
            mode_text(from, out, &mut index);
            put_str(out, &mut index, " > ");
            mode_text(to, out, &mut index);
        }
        Event::Error(code) => error_text(code, out, &mut index),
        _ => put_str(out, &mut index, "Event"),
    }
    text(out, index)
}

fn wake_text(reason: WakeReason, out: &mut [u8; TEXT_LEN], index: &mut usize) {
    match reason {
        WakeReason::Alarm => put_str(out, index, "Alarm"),
        WakeReason::Timer => put_str(out, index, "Woke: Timer"),
        WakeReason::Button => put_str(out, index, "Woke: Button"),
        WakeReason::Motion => put_str(out, index, "Woke: Motion"),
        WakeReason::Usb => put_str(out, index, "Woke: USB"),
        WakeReason::Unknown => put_str(out, index, "Woke: Unknown"),
        _ => put_str(out, index, "Woke: Unknown"),
    }
}

fn battery_text(millivolts: Millivolts, out: &mut [u8; TEXT_LEN], index: &mut usize) {
    put_str(out, index, "Battery ");
    let value = millivolts.value();
    put_u16(out, index, value / 1_000);
    out[*index] = b'.';
    *index += 1;
    put_two(out, index, ((value % 1_000) / 10) as u8);
    out[*index] = b'V';
    *index += 1;
}

fn mode_text(mode: Mode, out: &mut [u8; TEXT_LEN], index: &mut usize) {
    match mode {
        Mode::Watch => put_str(out, index, "Watch"),
        Mode::Desk => put_str(out, index, "Desk"),
    }
}

fn error_text(code: u16, out: &mut [u8; TEXT_LEN], index: &mut usize) {
    if let Some(label) = error_label(code) {
        put_str(out, index, label);
    } else {
        put_str(out, index, "Error ");
        put_u16(out, index, code);
    }
}

fn error_label(code: u16) -> Option<&'static str> {
    match code {
        1 => Some("Power fail"),
        2 => Some("Storage full"),
        3 => Some("Storage corrupt"),
        4 => Some("Bad config"),
        5 => Some("Hardware"),
        6 => Some("Timeout"),
        7 => Some("Unsupported"),
        8 => Some("Bad input"),
        9 => Some("Network"),
        10 => Some("Not ready"),
        11 => Some("Bad clock"),
        12 => Some("Not found"),
        _ => None,
    }
}

fn put_str(out: &mut [u8], index: &mut usize, value: &str) {
    for byte in value.as_bytes() {
        if *index >= out.len() {
            return;
        }
        out[*index] = *byte;
        *index += 1;
    }
}

fn put_u16(out: &mut [u8], index: &mut usize, value: u16) {
    let mut digits = [0u8; 5];
    let mut len = 0usize;
    let mut value = value;
    if value == 0 {
        digits[0] = b'0';
        len = 1;
    } else {
        while value > 0 && len < digits.len() {
            digits[len] = b'0' + (value % 10) as u8;
            value /= 10;
            len += 1;
        }
    }
    for digit in digits[..len].iter().rev() {
        if *index >= out.len() {
            return;
        }
        out[*index] = *digit;
        *index += 1;
    }
}

fn put_two(out: &mut [u8], index: &mut usize, value: u8) {
    let value = value.min(99);
    if *index + 1 >= out.len() {
        return;
    }
    out[*index] = b'0' + value / 10;
    out[*index + 1] = b'0' + value % 10;
    *index += 2;
}

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(timestamp: u64, event: Event) -> LogEntry {
        LogEntry::new(Timestamp::from_secs(timestamp), event)
    }

    #[test]
    fn time_text_includes_date_and_zero_is_blank() {
        let mut buffer = [0u8; 12];
        assert_eq!(
            time_text(
                Timestamp::from_secs(
                    DateTime::new(2026, 10, 18, 7, 5, 0)
                        .to_timestamp()
                        .unwrap()
                        .as_secs()
                ),
                &mut buffer
            ),
            "10-18 07:05"
        );
        assert_eq!(time_text(Timestamp::from_secs(0), &mut buffer), "--:--");
    }

    #[test]
    fn event_text_covers_every_event() {
        let mut buffer = [0u8; TEXT_LEN];
        assert_eq!(
            event_text(&entry(1, Event::Wake(WakeReason::Motion)), &mut buffer),
            "Woke: Motion"
        );
        assert_eq!(
            event_text(&entry(1, Event::Wake(WakeReason::Alarm)), &mut buffer),
            "Alarm"
        );
        assert_eq!(
            event_text(
                &entry(1, Event::Battery(Millivolts::new(3_812))),
                &mut buffer
            ),
            "Battery 3.81V"
        );
        assert_eq!(
            event_text(
                &entry(
                    1,
                    Event::ModeSwitch {
                        from: Mode::Watch,
                        to: Mode::Desk
                    }
                ),
                &mut buffer
            ),
            "Watch > Desk"
        );
        assert_eq!(
            event_text(&entry(1, Event::Error(10)), &mut buffer),
            "Not ready"
        );
        assert_eq!(
            event_text(&entry(1, Event::Error(99)), &mut buffer),
            "Error 99"
        );
    }
}
