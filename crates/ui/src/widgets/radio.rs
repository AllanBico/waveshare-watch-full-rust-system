//! Radio buttons.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{measure_text, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, draw_start, frame, shrink};
use super::{Widget, WidgetKind};

/// Props for one radio choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RadioProps<'a> {
    /// Whether this choice is selected.
    pub selected: bool,
    /// Row label.
    pub label: &'a str,
}

/// Measures a radio row.
pub(crate) fn measure(
    props: &RadioProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let label = measure_text(props.label, &theme.type_scale.body, 0, cache);
    let inset = super::checkbox::inset(theme, theme.spacing.md);
    let width = if available == UNBOUNDED {
        240
    } else {
        available
    };
    Size::new(
        width,
        theme.min_touch_px.max(label.height.saturating_add(inset)),
    )
}

/// Paints a radio row.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Radio(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.md);
    let size = 28;
    let choice = Region::new(
        region.x,
        region.y + centered_offset(region.height, size),
        size.min(region.width),
        size.min(region.height),
    );
    paint_choice(painter, choice, props.selected, theme, colors.content);
    let label_region = Region::new(
        choice.right().saturating_add(theme.spacing.md),
        region.y,
        region
            .right()
            .saturating_sub(choice.right().saturating_add(theme.spacing.md)),
        region.height,
    );
    draw_start(
        painter,
        props.label,
        &theme.type_scale.body,
        label_region,
        colors.content,
    );
}

fn paint_choice(
    painter: &mut Painter<'_>,
    region: Region,
    selected: bool,
    theme: &Theme,
    active: crate::luma::Luma,
) {
    let radius = region.height / 2;
    let border = theme.borders.thick.max(1);
    painter.fill_rounded_rect(region, radius, active);
    painter.fill_rounded_rect(
        shrink(region, border),
        radius.saturating_sub(border),
        theme.tones.background,
    );
    if selected {
        let dot = shrink(region, region.height / 4);
        painter.fill_rounded_rect(dot, dot.height / 2, active);
    }
}
