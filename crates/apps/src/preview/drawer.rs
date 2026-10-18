//! The preview app drawer: a full-page, three-column app grid.

use ui::{
    Align, IconButtonProps, IconId, Insets, LabelProps, Length, TextAlign, TextRole, TileProps,
    Widget, WidgetKind,
};

use crate::view::{
    background, column, fill_node, fit_node, grid, leaf, node, row, spacer, BuildError, Builder,
};

const BG_ID: u16 = 100;
const TITLE_ID: u16 = 101;
const CLOSE_ID: u16 = 102;
const GRID_ID: u16 = 103;

/// Settings tile.
pub const SETTINGS_ID: u16 = 110;
/// Alarms tile.
pub const ALARMS_ID: u16 = 111;
/// Gallery tile.
pub const GALLERY_ID: u16 = 112;

/// Whether `id` belongs to the drawer.
pub const fn owns(id: u16) -> bool {
    id >= BG_ID && id < 200
}

/// Whether `id` is the close control.
pub const fn is_close(id: u16) -> bool {
    id == CLOSE_ID
}

pub(super) fn build<'a>(view: &mut Builder<'_, 'a>) -> Result<usize, BuildError> {
    let theme = view.theme;
    let title = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Apps",
            role: TextRole::Label,
            align: TextAlign::Start,
            icon: None,
        }),
    );
    let close = Widget::new(
        CLOSE_ID,
        WidgetKind::IconButton(IconButtonProps {
            icon: IconId::CaretDown,
            label: "Close",
        }),
    );
    let settings = tile(SETTINGS_ID, "Settings", IconId::GearSix);
    let alarms = tile(ALARMS_ID, "Alarms", IconId::Bell);
    let gallery = tile(GALLERY_ID, "Gallery", IconId::SquaresFour);

    let content_width = view.viewport.width.saturating_sub(32);
    let cell = content_width.saturating_sub(16) / 3;
    let title_node = fill_node(&title, theme, view.cache, content_width);
    let close_node = fit_node(&close, theme, view.cache);
    let settings_node = node(&settings, theme, view.cache, Length::Fill(1), cell);
    let alarms_node = node(&alarms, theme, view.cache, Length::Fill(1), cell);
    let gallery_node = node(&gallery, theme, view.cache, Length::Fill(1), cell);
    view.push(background(BG_ID))?;
    for widget in [title, close, settings, alarms, gallery] {
        view.push(widget)?;
    }

    let header_children = [title_node, close_node];
    let header = row(104, &header_children, theme.spacing.sm, Align::Center);
    let grid_children = [settings_node, alarms_node, gallery_node];
    let apps = grid(
        GRID_ID,
        &grid_children,
        3,
        theme.spacing.sm,
        theme.spacing.sm,
    );
    let content_children = [
        header,
        spacer(105, Length::Fill(1), Length::Fill(1)),
        apps,
        spacer(106, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(107, &content_children, theme.spacing.sm, Align::Start)
        .with_padding(Insets::symmetric(16, 12));
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), content];
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

fn tile(id: u16, label: &'static str, icon: IconId) -> Widget<'static> {
    Widget::new(id, WidgetKind::Tile(TileProps { icon, label }))
}
