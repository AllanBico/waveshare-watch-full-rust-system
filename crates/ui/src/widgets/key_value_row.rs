//! Label-and-value data rows.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors};
use super::{Widget, WidgetKind};

const UNBOUNDED_WIDTH: u16 = 240;

/// Props for a label-and-value row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyValueRowProps<'a> {
    /// Left-hand label.
    pub key: &'a str,
    /// Right-hand value.
    pub value: &'a str,
}

/// Measures a key-value row.
pub(crate) fn measure(
    _props: &KeyValueRowProps<'_>,
    theme: &Theme,
    _cache: &mut TextCache,
    available: u16,
) -> Size {
    let inset = theme.spacing.sm.saturating_mul(2);
    let line = theme
        .type_scale
        .caption
        .line_height_px
        .max(theme.type_scale.numeric_small.line_height_px);
    Size::new(
        if available == UNBOUNDED {
            UNBOUNDED_WIDTH
        } else {
            available
        },
        line.saturating_add(inset),
    )
}

/// Paints a key-value row.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::KeyValueRow(props) = &widget.kind else {
        return;
    };
    let colors = colors(theme, widget.state);
    let region = Region::new(
        rect.x.saturating_add(theme.spacing.sm),
        rect.y,
        rect.width
            .saturating_sub(theme.spacing.sm.saturating_mul(2)),
        rect.height,
    );
    let value = measure_uncached(props.value, &theme.type_scale.numeric_small, region.width);
    let value_x = region.right().saturating_sub(value.width);
    paint_key(painter, props.key, region, value_x, theme);
    paint_value(painter, props.value, region, theme, colors.content);
}

fn paint_key(painter: &mut Painter<'_>, key: &str, region: Region, value_x: u16, theme: &Theme) {
    let style = theme.type_scale.caption;
    let metrics = measure_uncached(key, &style, region.width);
    draw_text(
        painter,
        key,
        &style,
        (
            region.x,
            region.y + centered_offset(region.height, metrics.height),
        ),
        value_x.saturating_sub(region.x),
        TextAlign::Start,
        theme.tones.text_muted,
    );
}

fn paint_value(
    painter: &mut Painter<'_>,
    value: &str,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let style = theme.type_scale.numeric_small;
    let metrics = measure_uncached(value, &style, region.width);
    let value_x = region.right().saturating_sub(metrics.width);
    draw_text(
        painter,
        value,
        &style,
        (
            value_x,
            region.y + centered_offset(region.height, metrics.height),
        ),
        region.right().saturating_sub(value_x),
        TextAlign::End,
        color,
    )
}
