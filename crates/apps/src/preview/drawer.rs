//! The preview app drawer: a bottom sheet with three columns.

use ui::{
    Align, IconId, Insets, LabelProps, Length, TextAlign, TextRole, TileProps, Widget, WidgetKind,
};

use crate::view::{background, column, fit_node, grid, leaf, node, spacer, BuildError, Builder};

/// Drawer panel height.
pub const HEIGHT: u16 = 260;

const PANEL_ID: u16 = 100;
const BG_ID: u16 = 101;
const TITLE_ID: u16 = 102;
const GRID_ID: u16 = 103;

/// Settings tile.
pub const SETTINGS_ID: u16 = 110;
/// Alarms tile.
pub const ALARMS_ID: u16 = 111;
/// Gallery tile.
pub const GALLERY_ID: u16 = 112;

/// Whether `id` belongs to the drawer.
pub const fn owns(id: u16) -> bool {
    id >= PANEL_ID && id < 200
}

pub(super) fn build<'a>(view: &mut Builder<'_, 'a>) -> Result<usize, BuildError> {
    let theme = view.theme;
    let title = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Apps",
            role: TextRole::Title,
            align: TextAlign::Start,
            icon: None,
        }),
    );
    let settings = tile(SETTINGS_ID, "Settings", IconId::GearSix);
    let alarms = tile(ALARMS_ID, "Alarms", IconId::Bell);
    let gallery = tile(GALLERY_ID, "Gallery", IconId::SquaresFour);

    let cell = view.viewport.width.saturating_sub(48 + 24) / 3;
    let title_node = fit_node(&title, theme, view.cache);
    let settings_node = node(&settings, theme, view.cache, Length::Fill(1), cell);
    let alarms_node = node(&alarms, theme, view.cache, Length::Fill(1), cell);
    let gallery_node = node(&gallery, theme, view.cache, Length::Fill(1), cell);
    view.push(background(BG_ID))?;
    for widget in [title, settings, alarms, gallery] {
        view.push(widget)?;
    }

    let grid_children = [settings_node, alarms_node, gallery_node];
    let dashboard = grid(
        GRID_ID,
        &grid_children,
        3,
        theme.spacing.md,
        theme.spacing.md,
    );
    let content_children = [
        title_node,
        spacer(104, Length::Fill(1), Length::Fixed(theme.spacing.md)),
        dashboard,
    ];
    let content = column(105, &content_children, theme.spacing.md, Align::Start)
        .with_padding(Insets::symmetric(24, 16));
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
        ui::Kind::Stack { align: Align::End },
        &root_children,
    );
    view.layout(&root)
}

fn tile(id: u16, label: &'static str, icon: IconId) -> Widget<'static> {
    Widget::new(id, WidgetKind::Tile(TileProps { icon, label }))
}
