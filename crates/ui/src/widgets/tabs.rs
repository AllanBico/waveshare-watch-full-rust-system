//! Horizontal tab strips.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors};
use super::{Widget, WidgetKind};

/// Props for a tab strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabsProps<'a> {
    /// Tab labels in order.
    pub labels: &'a [&'a str],
    /// Selected tab index.
    pub selected: u8,
}

/// Measures a tab strip.
pub(crate) fn measure(
    props: &TabsProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let mut width = 0u16;
    for label in props.labels {
        width = width.max(measure_text(label, &theme.type_scale.label, 0, cache).width);
    }
    let width = if available == UNBOUNDED {
        width.saturating_mul(props.labels.len() as u16).max(160)
    } else {
        available
    };
    Size::new(
        width,
        theme
            .min_touch_px
            .min(56)
            .max(theme.type_scale.label.line_height_px),
    )
}

/// Paints a tab strip.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Tabs(props) = &widget.kind else {
        return;
    };
    if props.labels.is_empty() {
        return;
    }
    let colors = colors(theme, widget.state);
    let inset = theme.spacing.sm;
    let region = Region::new(
        rect.x.saturating_add(inset),
        rect.y,
        rect.width.saturating_sub(inset.saturating_mul(2)),
        rect.height,
    );
    let count = props.labels.len() as u16;
    let cell_width = region.width / count;
    let selected = usize::from(props.selected).min(props.labels.len() - 1);
    for (index, label) in props.labels.iter().enumerate() {
        let x = region
            .x
            .saturating_add(cell_width.saturating_mul(index as u16));
        let cell = Region::new(x, region.y, cell_width, region.height);
        paint_cell(
            painter,
            label,
            cell,
            theme,
            colors.content,
            index == selected,
        );
    }
    painter.fill_rect(
        Region::new(region.x, rect.bottom().saturating_sub(1), region.width, 1),
        theme.tones.border,
    );
}

fn paint_cell(
    painter: &mut Painter<'_>,
    label: &str,
    cell: Region,
    theme: &Theme,
    active: crate::luma::Luma,
    is_selected: bool,
) {
    let color = if is_selected {
        active
    } else {
        theme.tones.text_muted
    };
    paint_label(painter, label, cell, theme, color);
    if is_selected {
        let underline = theme.borders.thick.max(2);
        painter.fill_rect(
            Region::new(
                cell.x,
                cell.bottom().saturating_sub(underline),
                cell.width,
                underline,
            ),
            color,
        );
    }
}

fn paint_label(
    painter: &mut Painter<'_>,
    label: &str,
    cell: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let style = theme.type_scale.label;
    let metrics = measure_uncached(label, &style, cell.width);
    let y = cell.y + centered_offset(cell.height, metrics.height);
    draw_text(
        painter,
        label,
        &style,
        (cell.x, y),
        cell.width,
        TextAlign::Center,
        color,
    );
}
