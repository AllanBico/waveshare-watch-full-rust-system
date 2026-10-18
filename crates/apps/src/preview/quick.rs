//! The preview quick settings: a top sheet with a two-column toggle grid.

use common::Mode;
use settings::Settings;
use ui::{
    Align, Insets, LabelProps, Length, SliderProps, TextAlign, TextRole, ToggleProps, Widget,
    WidgetKind,
};

use crate::view::{background, column, crown, fill_node, grid, leaf, node, BuildError, Builder};

/// Quick-settings panel height.
pub const HEIGHT: u16 = 300;

const PANEL_ID: u16 = 200;
const BG_ID: u16 = 201;
const TITLE_ID: u16 = 202;
const GRID_ID: u16 = 203;
const CROWN_ID: u16 = 204;
const CONTENT_ID: u16 = 205;
const BODY_ID: u16 = 206;

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

/// Whether `id` belongs to quick settings.
pub const fn owns(id: u16) -> bool {
    id >= PANEL_ID && id < 300
}

pub(super) fn build<'a>(
    settings: &Settings,
    mode: Mode,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let content_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.content_inset * 2);
    let cell = content_width.saturating_sub(theme.spacing.sm) / 2;
    let title = title();
    let toggles = toggles(settings);
    let brightness = brightness(settings, mode);
    let title_node = fill_node(&title, theme, view.cache, content_width);
    let toggle_nodes =
        toggles.map(|widget| node(&widget, theme, view.cache, Length::Fill(1), cell));
    let brightness_node = node(
        &brightness,
        theme,
        view.cache,
        Length::Fill(1),
        content_width,
    );

    view.push(background(BG_ID))?;
    view.push(title)?;
    for widget in toggles {
        view.push(widget)?;
    }
    view.push(brightness)?;

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let toggles_grid = grid(
        GRID_ID,
        &toggle_nodes,
        2,
        theme.spacing.md,
        theme.spacing.md,
    );
    layout_panel(view, crown_node, toggles_grid, brightness_node)
}

fn layout_panel<'a, 'n>(
    view: &mut Builder<'_, 'a>,
    crown_node: ui::Node<'n>,
    toggles: ui::Node<'n>,
    brightness: ui::Node<'n>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let content_children = [toggles, brightness];
    let content = column(
        CONTENT_ID,
        &content_children,
        theme.spacing.sm,
        Align::Start,
    )
    .with_padding(Insets::new(
        0,
        theme.shape.content_inset,
        theme.shape.content_inset,
        theme.shape.content_inset,
    ));
    let body_children = [crown_node, content];
    let body = column(BODY_ID, &body_children, 0, Align::Start);
    let panel_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), body];
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

fn title() -> Widget<'static> {
    Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Quick settings",
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    )
}

fn toggles(settings: &Settings) -> [Widget<'static>; 4] {
    [
        toggle(WIFI_ID, "Wi-Fi", settings.connectivity.wifi),
        toggle(BLUETOOTH_ID, "BT", settings.connectivity.bluetooth),
        toggle(DND_ID, "DND", settings.sound.is_dnd),
        toggle(SYNC_ID, "Sync", settings.connectivity.sync),
    ]
}

fn brightness(settings: &Settings, mode: Mode) -> Widget<'static> {
    Widget::new(
        BRIGHTNESS_ID,
        WidgetKind::Slider(SliderProps {
            value: settings.brightness(mode).value(),
            label: "Brightness",
        }),
    )
}

fn toggle(id: u16, label: &'static str, checked: bool) -> Widget<'static> {
    Widget::new(id, WidgetKind::Toggle(ToggleProps { checked, label }))
}
