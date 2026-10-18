//! The preview quick settings: a top sheet with a two-column toggle grid.

use ui::{
    Align, Insets, LabelProps, Length, SliderProps, TextAlign, TextRole, ToggleProps, Widget,
    WidgetKind,
};

use crate::view::{background, column, fit_node, grid, leaf, node, BuildError, Builder};

/// Quick-settings panel height.
pub const HEIGHT: u16 = 300;

const PANEL_ID: u16 = 200;
const BG_ID: u16 = 201;
const TITLE_ID: u16 = 202;
const GRID_ID: u16 = 203;

/// Wi-Fi toggle.
pub const WIFI_ID: u16 = 210;
/// Bluetooth toggle.
pub const BLUETOOTH_ID: u16 = 211;
/// Do-not-disturb toggle.
pub const DND_ID: u16 = 212;
/// Sync toggle.
pub const SYNC_ID: u16 = 213;
/// Brightness slider.
pub const BRIGHTNESS_ID: u16 = 214;

/// Visual state for the quick-settings sheet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuickState {
    /// Wi-Fi enabled.
    pub wifi: bool,
    /// Bluetooth enabled.
    pub bluetooth: bool,
    /// Do-not-disturb enabled.
    pub dnd: bool,
    /// Sync enabled.
    pub sync: bool,
    /// Brightness from 0 to 100.
    pub brightness: u8,
}

impl QuickState {
    /// Default preview state.
    pub const fn new() -> Self {
        Self {
            wifi: true,
            bluetooth: false,
            dnd: false,
            sync: true,
            brightness: 70,
        }
    }
}

impl Default for QuickState {
    fn default() -> Self {
        Self::new()
    }
}

/// Whether `id` belongs to quick settings.
pub const fn owns(id: u16) -> bool {
    id >= PANEL_ID && id < 300
}

pub(super) fn build<'a>(
    state: &QuickState,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let title = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Quick settings",
            role: TextRole::Label,
            align: TextAlign::Start,
            icon: None,
        }),
    );
    let wifi = toggle(WIFI_ID, "Wi-Fi", state.wifi);
    let bluetooth = toggle(BLUETOOTH_ID, "BT", state.bluetooth);
    let dnd = toggle(DND_ID, "DND", state.dnd);
    let sync = toggle(SYNC_ID, "Sync", state.sync);
    let brightness = Widget::new(
        BRIGHTNESS_ID,
        WidgetKind::Slider(SliderProps {
            value: state.brightness.min(100),
            label: "Brightness",
        }),
    );

    let content_width = view.viewport.width.saturating_sub(48);
    let cell = content_width.saturating_sub(theme.spacing.sm) / 2;
    let title_node = fit_node(&title, theme, view.cache);
    let wifi_node = node(&wifi, theme, view.cache, Length::Fill(1), cell);
    let bluetooth_node = node(&bluetooth, theme, view.cache, Length::Fill(1), cell);
    let dnd_node = node(&dnd, theme, view.cache, Length::Fill(1), cell);
    let sync_node = node(&sync, theme, view.cache, Length::Fill(1), cell);
    let brightness_node = node(
        &brightness,
        theme,
        view.cache,
        Length::Fill(1),
        content_width,
    );
    view.push(background(BG_ID))?;
    for widget in [title, wifi, bluetooth, dnd, sync, brightness] {
        view.push(widget)?;
    }

    let grid_children = [wifi_node, bluetooth_node, dnd_node, sync_node];
    let toggles = grid(
        GRID_ID,
        &grid_children,
        2,
        theme.spacing.md,
        theme.spacing.md,
    );
    let content_children = [title_node, toggles, brightness_node];
    let content = column(207, &content_children, theme.spacing.sm, Align::Start)
        .with_padding(Insets::symmetric(16, 12));
    let panel_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), content];
    let panel = ui::Node::container(
        PANEL_ID,
        Length::Fill(1),
        Length::Fixed(HEIGHT),
        ui::Kind::Stack {
            align: Align::Start,
        },
        &panel_children,
    );
    let root_children = [panel];
    let root = ui::Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        ui::Kind::Stack {
            align: Align::Start,
        },
        &root_children,
    );
    view.layout(&root)
}

fn toggle<'a>(id: u16, label: &'a str, checked: bool) -> Widget<'a> {
    Widget::new(id, WidgetKind::Toggle(ToggleProps { checked, label }))
}
