//! Controls, values, and structure pages.

use ui::{CardTone, DividerOrientation, IconId, IconSize, SettingsTrailing, Status, Widget};

use super::super::WIDGET_BASE;
use super::{
    badge, button, card, checkbox, divider, icon, icon_button, input, key_value, list_item,
    progress, radio, ring, segmented, settings_row, slider, spinner, status, stepper, tabs, tile,
    toggle,
};

const BASE: u16 = WIDGET_BASE;

const SEGMENT_LABELS: [&str; 3] = ["Day", "Week", "Month"];
const TAB_LABELS: [&str; 3] = ["Home", "Apps", "Settings"];

const ACTIONS: [Widget<'static>; 4] = [
    button(BASE, "Continue", Some(IconId::Check)),
    icon_button(BASE + 1, IconId::GearSix, "Settings"),
    tile(BASE + 2, "Gallery", IconId::SquaresFour),
    status(BASE + 3, "Connected", Status::Ok),
];

const SELECTION: [Widget<'static>; 4] = [
    toggle(BASE, "Wi-Fi", true),
    checkbox(BASE + 1, "Sync", true),
    radio(BASE + 2, "Option A", true),
    segmented(BASE + 3, &SEGMENT_LABELS, 1),
];

const VALUES: [Widget<'static>; 4] = [
    slider(BASE, "Volume", 60),
    stepper(BASE + 1, "Alarm", 7, "h"),
    progress(BASE + 2, "Sync", 60),
    spinner(BASE + 3),
];

const DATA: [Widget<'static>; 4] = [
    ring(BASE, "Battery", 60),
    key_value(BASE + 1, "Steps", "8,420"),
    icon(BASE + 2, IconId::Heart, IconSize::Lg),
    divider(BASE + 3, DividerOrientation::Horizontal),
];

const INPUTS: [Widget<'static>; 4] = [
    input(BASE, "Note", Some(4)),
    settings_row(
        BASE + 1,
        "Bluetooth",
        "Paired",
        Some(IconId::Bluetooth),
        SettingsTrailing::Toggle(true),
    ),
    settings_row(
        BASE + 2,
        "Sync",
        "Last 07:00",
        Some(IconId::ArrowClockwise),
        SettingsTrailing::Value("Idle"),
    ),
    tabs(BASE + 3, &TAB_LABELS, 1),
];

const STRUCTURE: [Widget<'static>; 5] = [
    list_item(BASE, "Notifications", "3 unread", Some(IconId::Bell)),
    card(BASE + 1, CardTone::Surface),
    badge(BASE + 2, "New"),
    divider(BASE + 3, DividerOrientation::Horizontal),
    settings_row(
        BASE + 4,
        "Display",
        "Theme and brightness",
        Some(IconId::Sun),
        SettingsTrailing::Chevron,
    ),
];

pub(super) fn widgets(page: u8) -> &'static [Widget<'static>] {
    match page {
        0 => &ACTIONS,
        1 => &SELECTION,
        2 => &VALUES,
        3 => &DATA,
        4 => &INPUTS,
        5 => &STRUCTURE,
        _ => &[],
    }
}
