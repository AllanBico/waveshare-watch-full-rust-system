//! The preview app drawer: a full-page, three-column app grid.

use ui::{Align, IconId, LabelProps, Length, TextAlign, TextRole, TileProps, Widget, WidgetKind};

use crate::view::{
    background, column, content_padding, crown, fill_node, grid, leaf, sized_node, spacer, stack,
    BuildError, Builder,
};

const BG_ID: u16 = 100;
const TITLE_ID: u16 = 101;
const GRID_ID: u16 = 103;
const CROWN_ID: u16 = 104;
const CONTENT_ID: u16 = 105;
const BODY_ID: u16 = 108;

/// Settings tile.
pub const SETTINGS_ID: u16 = 110;
/// Alarms tile.
pub const ALARMS_ID: u16 = 111;
/// Gallery tile.
pub const GALLERY_ID: u16 = 112;
/// Calibration tile.
pub const CALIBRATE_ID: u16 = 113;

/// Whether `id` belongs to the drawer.
pub const fn owns(id: u16) -> bool {
    id >= BG_ID && id < 200
}

pub(super) fn build<'a>(view: &mut Builder<'_, 'a>) -> Result<usize, BuildError> {
    let theme = view.theme;
    let title = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Apps",
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    );
    let settings = tile(SETTINGS_ID, "Settings", IconId::GearSix);
    let alarms = tile(ALARMS_ID, "Alarms", IconId::Bell);
    let gallery = tile(GALLERY_ID, "Gallery", IconId::SquaresFour);
    let calibrate = tile(CALIBRATE_ID, "Calibrate", IconId::PencilSimple);

    let content_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.content_inset * 2);
    let tile_height = theme.min_touch_px.saturating_mul(2);
    let title_node = fill_node(&title, theme, view.cache, content_width);
    let settings_node = sized_node(SETTINGS_ID, Length::Fill(1), tile_height);
    let alarms_node = sized_node(ALARMS_ID, Length::Fill(1), tile_height);
    let gallery_node = sized_node(GALLERY_ID, Length::Fill(1), tile_height);
    let calibrate_node = sized_node(CALIBRATE_ID, Length::Fill(1), tile_height);
    view.push(background(BG_ID))?;
    for widget in [title, settings, alarms, gallery, calibrate] {
        view.push(widget)?;
    }

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let grid_children = [settings_node, alarms_node, gallery_node, calibrate_node];
    let apps = grid(
        GRID_ID,
        &grid_children,
        2,
        theme.spacing.sm,
        theme.spacing.sm,
    );
    let content_children = [
        spacer(106, Length::Fill(1), Length::Fill(1)),
        apps,
        spacer(107, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(
        CONTENT_ID,
        &content_children,
        theme.spacing.sm,
        Align::Start,
    )
    .with_padding(content_padding(theme));
    let body_children = [crown_node, content];
    let body = column(BODY_ID, &body_children, 0, Align::Start);
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), body];
    let root = stack(0, &root_children, Align::Start);
    view.layout(&root)
}

fn tile(id: u16, label: &'static str, icon: IconId) -> Widget<'static> {
    Widget::new(id, WidgetKind::Tile(TileProps { icon, label }))
}
