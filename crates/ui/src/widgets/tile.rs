//! Square app and shortcut tiles.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, frame, shrink};
use super::{Widget, WidgetKind};

/// Props for an icon-and-label tile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TileProps<'a> {
    /// Tile icon.
    pub icon: IconId,
    /// Tile label.
    pub label: &'a str,
}

/// Measures a tile.
pub(crate) fn measure(
    props: &TileProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let label = measure_text(props.label, &theme.type_scale.caption, available, cache);
    let inset = theme.spacing.sm.saturating_mul(2);
    let width = label.width.saturating_add(inset).max(theme.min_touch_px);
    let width = if available == UNBOUNDED {
        width
    } else {
        available
    };
    let height = theme
        .icons
        .md
        .saturating_add(theme.spacing.sm)
        .saturating_add(label.height)
        .saturating_add(inset)
        .max(theme.min_touch_px);
    Size::new(width, height)
}

/// Paints a tile.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Tile(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.lg);
    let colors = colors(theme, widget.state);
    let region = shrink(rect, theme.borders.thick + 1 + theme.spacing.sm);
    let icon = theme.icons.md.min(region.width).min(region.height);
    let text = measure_uncached(props.label, &theme.type_scale.caption, region.width);
    let total = icon
        .saturating_add(theme.spacing.sm)
        .saturating_add(text.height);
    let top = region.y + centered_offset(region.height, total);
    let x = region.x + centered_offset(region.width, icon);
    draw_icon(painter, props.icon, icon, (x, top), colors.content);
    draw_text(
        painter,
        props.label,
        &theme.type_scale.caption,
        (region.x, top + icon + theme.spacing.sm),
        region.width,
        TextAlign::Center,
        colors.content,
    );
}
