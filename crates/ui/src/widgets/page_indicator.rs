//! Page position dots.

use common::Region;

use crate::layout::Size;
use crate::painter::Painter;
use crate::text::TextCache;
use crate::theme::Theme;

use super::paint::{colors, shrink};
use super::{Widget, WidgetKind};

const DOT_PX: u16 = 8;

/// Props for a page indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PageIndicatorProps {
    /// Number of pages.
    pub count: u8,
    /// Selected page index.
    pub selected: u8,
}

/// Measures a page indicator.
pub(crate) fn measure(
    props: &PageIndicatorProps,
    theme: &Theme,
    _cache: &mut TextCache,
    _available: u16,
) -> Size {
    let gap = theme.spacing.xs;
    let count = u16::from(props.count);
    let width = DOT_PX
        .saturating_mul(count)
        .saturating_add(gap.saturating_mul(count.saturating_sub(1)));
    Size::new(width.max(DOT_PX), DOT_PX)
}

/// Paints a page indicator.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::PageIndicator(props) = &widget.kind else {
        return;
    };
    if props.count == 0 {
        return;
    }
    let colors = colors(theme, widget.state);
    let gap = theme.spacing.xs;
    let count = u16::from(props.count.max(1));
    let total = DOT_PX
        .saturating_mul(count)
        .saturating_add(gap.saturating_mul(count.saturating_sub(1)));
    let start = rect.x + rect.width.saturating_sub(total) / 2;
    let top = rect.y + rect.height.saturating_sub(DOT_PX) / 2;
    let selected = usize::from(props.selected);
    for index in 0..count {
        let x = start.saturating_add(index.saturating_mul(DOT_PX.saturating_add(gap)));
        let dot = Region::new(x, top, DOT_PX.min(rect.width), DOT_PX.min(rect.height));
        if usize::from(index) == selected {
            painter.fill_rounded_rect(dot, DOT_PX / 2, colors.content);
        } else {
            painter.fill_rounded_rect(dot, DOT_PX / 2, theme.tones.border);
            painter.fill_rounded_rect(
                shrink(dot, theme.borders.thin.max(1)),
                DOT_PX / 2,
                theme.tones.background,
            );
        }
    }
}
