//! Typography, overlay, and state pages.

use ui::{IconId, MenuItem, TextAlign, TextRole, Widget};

use super::super::WIDGET_BASE;
use super::{confirm, dialog, empty, error, label, loading, menu, scaffold, toast};

const BASE: u16 = WIDGET_BASE;

const TEXT: [Widget<'static>; 4] = [
    label(BASE, "Title label", TextRole::Title, TextAlign::Center),
    label(BASE + 1, "Body label", TextRole::Body, TextAlign::Start),
    label(
        BASE + 2,
        "Caption label",
        TextRole::Caption,
        TextAlign::Start,
    ),
    scaffold(BASE + 3, "Screen scaffold", "Reusable titled screen body."),
];

const MENU_ITEMS: [MenuItem<'static>; 2] = [
    MenuItem {
        label: "Open",
        icon: Some(IconId::CaretRight),
    },
    MenuItem {
        label: "Delete",
        icon: Some(IconId::Trash),
    },
];

const OVERLAYS: [Widget<'static>; 3] = [
    dialog(BASE, "Delete?", "This cannot be undone."),
    toast(BASE + 1, "Saved", Some(IconId::CheckCircle)),
    menu(BASE + 2, &MENU_ITEMS),
];

const STATES: [Widget<'static>; 2] = [
    empty(
        BASE,
        IconId::Bell,
        "No alarms",
        "Add an alarm to get started.",
    ),
    loading(BASE + 1, "Loading"),
];

const ERROR: [Widget<'static>; 1] = [error(
    BASE,
    "Sync failed",
    "Check your connection.",
    "Retry",
)];

const CONFIRM: [Widget<'static>; 1] = [confirm(
    BASE,
    "Delete?",
    "This cannot be undone.",
    "Delete",
    "Cancel",
)];

pub(super) fn widgets(page: u8) -> &'static [Widget<'static>] {
    match page {
        6 => &TEXT,
        7 => &OVERLAYS,
        8 => &STATES,
        9 => &ERROR,
        10 => &CONFIRM,
        _ => &[],
    }
}
