//! Segmented controls for small exclusive choices.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, frame, on};
use super::{Widget, WidgetKind};

const UNBOUNDED_MIN_WIDTH: u16 = 160;
const MAX_HEIGHT: u16 = 56;

/// Props for a segmented control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SegmentedControlProps<'a> {
    /// Segment labels in order.
    pub labels: &'a [&'a str],
    /// Selected segment index.
    pub selected: u8,
}

/// Measures a segmented control.
pub(crate) fn measure(
    props: &SegmentedControlProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let mut width = 0u16;
    for label in props.labels {
        width = width.max(measure_text(label, &theme.type_scale.label, 0, cache).width);
    }
    let width = if available == UNBOUNDED {
        width
            .saturating_mul(props.labels.len() as u16)
            .max(UNBOUNDED_MIN_WIDTH)
    } else {
        available
    };
    Size::new(width, theme.min_touch_px.min(MAX_HEIGHT))
}

/// Paints a segmented control.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::SegmentedControl(props) = &widget.kind else {
        return;
    };
    if props.labels.is_empty() {
        return;
    }
    frame(painter, rect, theme, widget.state, theme.radii.pill);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.xs);
    let count = props.labels.len() as u16;
    let cell_width = region.width / count;
    let selected = usize::from(props.selected).min(props.labels.len() - 1);
    for (index, label) in props.labels.iter().enumerate() {
        let x = region
            .x
            .saturating_add(cell_width.saturating_mul(index as u16));
        let cell = Region::new(x, region.y, cell_width, region.height);
        if index == selected {
            painter.fill_rounded_rect(cell, theme.radii.pill, colors.content);
            paint_label(painter, label, cell, theme, on(theme, colors.content));
        } else {
            paint_label(painter, label, cell, theme, theme.tones.text_muted);
        }
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
