//! Settings rows, labels, and the next value for each row.

use common::{Mode, Percent};
use settings::{SettingId, SettingValue, Settings};
use ui::{IconId, SettingsRowProps, SettingsTrailing, Widget, WidgetKind};

const ROW_BASE: u16 = 820;

/// Time format row.
pub(super) const TIME_FORMAT_ID: u16 = ROW_BASE;
/// Date format row.
pub(super) const DATE_FORMAT_ID: u16 = ROW_BASE + 1;
/// Brightness row.
pub(super) const BRIGHTNESS_ID: u16 = ROW_BASE + 2;
/// Sleep timeout row.
pub(super) const SLEEP_TIMEOUT_ID: u16 = ROW_BASE + 3;

const SLEEP_STEPS: [u16; 6] = [5, 10, 15, 30, 60, 120];

/// Builds the four watch settings rows.
pub(super) fn rows<'a>(
    settings: &Settings,
    brightness: &'a str,
    sleep: &'a str,
) -> [Widget<'a>; 4] {
    [
        row(
            TIME_FORMAT_ID,
            "Time format",
            IconId::Clock,
            SettingsTrailing::Value(settings.display.time_format.label()),
        ),
        row(
            DATE_FORMAT_ID,
            "Date format",
            IconId::List,
            SettingsTrailing::Value(settings.display.date_format.label()),
        ),
        row(
            BRIGHTNESS_ID,
            "Brightness",
            IconId::Sun,
            SettingsTrailing::Value(brightness),
        ),
        row(
            SLEEP_TIMEOUT_ID,
            "Sleep timeout",
            IconId::Power,
            SettingsTrailing::Value(sleep),
        ),
    ]
}

/// Computes the next value for a tapped row.
pub(super) fn next_value(
    id: u16,
    settings: &Settings,
    mode: Mode,
) -> Option<(SettingId, SettingValue)> {
    let change = match id {
        TIME_FORMAT_ID => (
            SettingId::DisplayTimeFormat,
            SettingValue::TimeFormat(settings.display.time_format.next()),
        ),
        DATE_FORMAT_ID => (
            SettingId::DisplayDateFormat,
            SettingValue::DateFormat(settings.display.date_format.next()),
        ),
        BRIGHTNESS_ID => (
            SettingId::brightness(mode),
            SettingValue::Percent(next_percent(settings.brightness(mode))),
        ),
        SLEEP_TIMEOUT_ID => {
            let cap = settings.power.awake_cap_secs;
            let current = settings.power.sleep_timeout_secs.min(cap);
            (
                SettingId::PowerSleepTimeout,
                SettingValue::Seconds(next_sleep(current, cap)),
            )
        }
        _ => return None,
    };
    Some(change)
}

/// Formats a short seconds label for the sleep row.
pub(super) fn seconds_text(seconds: u16, out: &mut [u8; 8]) -> &str {
    let mut digits = [0u8; 5];
    let mut len = 0usize;
    let mut value = seconds;
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
    for index in 0..len {
        out[index] = digits[len - 1 - index];
    }
    out[len] = b's';
    text(out, len + 1)
}

fn row<'a>(id: u16, label: &'a str, icon: IconId, trailing: SettingsTrailing<'a>) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::SettingsRow(SettingsRowProps {
            title: label,
            subtitle: "",
            icon: Some(icon),
            trailing,
            compact: true,
        }),
    )
}

fn next_percent(value: Percent) -> Percent {
    Percent::new(((value.value() / 10) + 1) % 11 * 10)
}

pub(super) fn next_sleep(current: u16, cap: u16) -> u16 {
    SLEEP_STEPS
        .iter()
        .copied()
        .find(|step| *step > current && *step <= cap)
        .unwrap_or(SLEEP_STEPS[0])
}

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}
