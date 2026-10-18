//! The persisted settings list.

use common::{Mode, Percent};
use settings::{SettingId, SettingValue, Settings};
use ui::{
    Align, Axis, IconId, Kind, Length, Node, SettingsRowProps, SettingsTrailing, TextAlign,
    TextRole, Widget, WidgetKind,
};

use crate::view::{
    background, column, content_padding, crown, fill_node, leaf, BuildError, Builder,
};

const BG_ID: u16 = 700;
const TITLE_ID: u16 = 702;
const CROWN_ID: u16 = 706;
const SCROLL_ID: u16 = 707;
const CONTENT_ID: u16 = 708;
const BODY_ID: u16 = 710;
const ROW_BASE: u16 = 720;
const ROW_COUNT: u16 = 9;

/// Largest scroll offset in pixels.
pub(super) const MAX_OFFSET: u16 = 536;

/// Clock format row.
pub const TIME_FORMAT_ID: u16 = ROW_BASE;
/// Theme row.
pub const THEME_ID: u16 = ROW_BASE + 1;
/// Brightness row.
pub const BRIGHTNESS_ID: u16 = ROW_BASE + 2;
/// Volume row.
pub const VOLUME_ID: u16 = ROW_BASE + 3;
/// Wi-Fi row.
pub const WIFI_ID: u16 = ROW_BASE + 4;
/// Bluetooth row.
pub const BLUETOOTH_ID: u16 = ROW_BASE + 5;
/// Do-not-disturb row.
pub const DND_ID: u16 = ROW_BASE + 6;
/// Sync row.
pub const SYNC_ID: u16 = ROW_BASE + 7;
/// Wake-on-motion row.
pub const WAKE_ID: u16 = ROW_BASE + 8;

/// Computes the next value for a tapped row.
pub(super) fn apply(id: u16, settings: &Settings, mode: Mode) -> Option<(SettingId, SettingValue)> {
    let change = match id {
        TIME_FORMAT_ID => (
            SettingId::DisplayTimeFormat,
            SettingValue::TimeFormat(settings.display.time_format.next()),
        ),
        THEME_ID => (
            SettingId::DisplayTheme,
            SettingValue::Theme(settings.display.theme.next()),
        ),
        BRIGHTNESS_ID => (
            SettingId::brightness(mode),
            SettingValue::Percent(next_percent(settings.brightness(mode))),
        ),
        VOLUME_ID => (
            SettingId::SoundVolume,
            SettingValue::Percent(next_percent(settings.sound.volume)),
        ),
        WIFI_ID => (
            SettingId::ConnectivityWifi,
            SettingValue::Bool(!settings.connectivity.wifi),
        ),
        BLUETOOTH_ID => (
            SettingId::ConnectivityBluetooth,
            SettingValue::Bool(!settings.connectivity.bluetooth),
        ),
        DND_ID => (
            SettingId::SoundDnd,
            SettingValue::Bool(!settings.sound.is_dnd),
        ),
        SYNC_ID => (
            SettingId::ConnectivitySync,
            SettingValue::Bool(!settings.connectivity.sync),
        ),
        WAKE_ID => (
            SettingId::PowerWakeOnMotion,
            SettingValue::Bool(!settings.power.wake_on_motion),
        ),
        _ => return None,
    };
    Some(change)
}

fn next_percent(value: Percent) -> Percent {
    Percent::new(((value.value() / 10) + 1) % 11 * 10)
}

pub(super) fn build<'a>(
    settings: &'a Settings,
    brightness: &'a str,
    volume: &'a str,
    offset: u16,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let title = title();
    let rows = rows(settings, brightness, volume);
    let title_node = fill_node(&title, theme, view.cache, view.viewport.width);
    let row_nodes = rows.map(|row| fill_node(&row, theme, view.cache, view.viewport.width));

    view.push(background(BG_ID))?;
    view.push(title)?;
    for row in rows {
        view.push(row)?;
    }

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let content = column(CONTENT_ID, &row_nodes, theme.spacing.xs, Align::Start)
        .with_padding(content_padding(theme));
    let scroll_children = [content];
    let scroll = Node::container(
        SCROLL_ID,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Scroll {
            axis: Axis::Vertical,
            offset,
        },
        &scroll_children,
    );
    let body_children = [crown_node, scroll];
    let body = column(BODY_ID, &body_children, 0, Align::Start);
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), body];
    let root = crate::view::stack(0, &root_children, Align::Start);
    view.layout(&root)
}

fn title<'a>() -> Widget<'a> {
    Widget::new(
        TITLE_ID,
        WidgetKind::Label(ui::LabelProps {
            text: "Settings",
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    )
}

fn rows<'a>(
    settings: &Settings,
    brightness: &'a str,
    volume: &'a str,
) -> [Widget<'a>; ROW_COUNT as usize] {
    let time = settings.display.time_format.label();
    [
        row(
            TIME_FORMAT_ID,
            "Time",
            "Clock format",
            IconId::Clock,
            SettingsTrailing::Value(time),
        ),
        row(
            THEME_ID,
            "Theme",
            "Appearance",
            IconId::Moon,
            SettingsTrailing::Value(settings.display.theme.label()),
        ),
        row(
            BRIGHTNESS_ID,
            "Brightness",
            "Display",
            IconId::Sun,
            SettingsTrailing::Value(brightness),
        ),
        row(
            VOLUME_ID,
            "Volume",
            "Sound",
            IconId::SpeakerHigh,
            SettingsTrailing::Value(volume),
        ),
        row(
            WIFI_ID,
            "Wi-Fi",
            "Connectivity",
            IconId::WifiHigh,
            SettingsTrailing::Toggle(settings.connectivity.wifi),
        ),
        row(
            BLUETOOTH_ID,
            "Bluetooth",
            "Connectivity",
            IconId::Bluetooth,
            SettingsTrailing::Toggle(settings.connectivity.bluetooth),
        ),
        row(
            DND_ID,
            "Do not disturb",
            "Sound",
            IconId::Moon,
            SettingsTrailing::Toggle(settings.sound.is_dnd),
        ),
        row(
            SYNC_ID,
            "Sync",
            "Connectivity",
            IconId::ArrowClockwise,
            SettingsTrailing::Toggle(settings.connectivity.sync),
        ),
        row(
            WAKE_ID,
            "Wake on motion",
            "Power",
            IconId::Power,
            SettingsTrailing::Toggle(settings.power.wake_on_motion),
        ),
    ]
}

fn row<'a>(
    id: u16,
    label: &'a str,
    subtitle: &'a str,
    icon: IconId,
    trailing: SettingsTrailing<'a>,
) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::SettingsRow(SettingsRowProps {
            title: label,
            subtitle,
            icon: Some(icon),
            trailing,
        }),
    )
}
