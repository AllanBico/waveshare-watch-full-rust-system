//! Inline activity spinners.

use common::Region;

use crate::layout::Size;
use crate::painter::Painter;
use crate::text::TextCache;
use crate::theme::Theme;

use super::paint::{colors, draw_dot};
use super::{Widget, WidgetKind};

const DOT_COUNT: u8 = 8;
const UNIT: i32 = 1024;
const DIAGONAL: i32 = 724;

const DOTS: [(i32, i32); 8] = [
    (0, -UNIT),
    (DIAGONAL, -DIAGONAL),
    (UNIT, 0),
    (DIAGONAL, DIAGONAL),
    (0, UNIT),
    (-DIAGONAL, DIAGONAL),
    (-UNIT, 0),
    (-DIAGONAL, -DIAGONAL),
];

/// Props for an inline spinner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpinnerProps {
    /// Bright-dot position, from 0 to 7.
    pub phase: u8,
}

/// Measures a spinner.
pub(crate) fn measure(
    _props: &SpinnerProps,
    theme: &Theme,
    _cache: &mut TextCache,
    _available: u16,
) -> Size {
    Size::new(theme.icons.md, theme.icons.md)
}

/// Paints a spinner.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Spinner(props) = &widget.kind else {
        return;
    };
    let colors = colors(theme, widget.state);
    let side = theme.icons.md.min(rect.width).min(rect.height);
    if side == 0 {
        return;
    }
    let center = (rect.x + rect.width / 2, rect.y + rect.height / 2);
    let radius = i32::from(side / 2).saturating_sub(i32::from(theme.spacing.xxs.max(1)));
    let dot_radius = theme.spacing.xxs.max(1);
    let active = usize::from(props.phase % DOT_COUNT);
    for (index, (unit_x, unit_y)) in DOTS.iter().enumerate() {
        let x = i32::from(center.0) + unit_x * radius / UNIT;
        let y = i32::from(center.1) + unit_y * radius / UNIT;
        let color = if index == active {
            colors.content
        } else {
            theme.tones.border
        };
        draw_dot(
            painter,
            (x.max(0) as u16, y.max(0) as u16),
            dot_radius,
            color,
        );
    }
}
