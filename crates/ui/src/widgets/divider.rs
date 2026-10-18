//! Horizontal and vertical dividers.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::TextCache;
use crate::theme::Theme;

use super::{Widget, WidgetKind};

const UNBOUNDED_WIDTH: u16 = 240;
const VERTICAL_HEIGHT: u16 = 64;

/// The direction a divider runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DividerOrientation {
    /// A line across the available width.
    Horizontal,
    /// A line down the available height.
    Vertical,
}

/// Props for a divider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DividerProps {
    /// Line direction.
    pub orientation: DividerOrientation,
}

/// Measures a divider.
pub(crate) fn measure(
    props: &DividerProps,
    theme: &Theme,
    _cache: &mut TextCache,
    available: u16,
) -> Size {
    match props.orientation {
        DividerOrientation::Horizontal => Size::new(
            if available == UNBOUNDED {
                UNBOUNDED_WIDTH
            } else {
                available
            },
            theme.spacing.sm,
        ),
        DividerOrientation::Vertical => Size::new(theme.spacing.sm, VERTICAL_HEIGHT),
    }
}

/// Paints a divider.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Divider(props) = &widget.kind else {
        return;
    };
    let thickness = theme.borders.thin.max(1);
    match props.orientation {
        DividerOrientation::Horizontal => {
            let y = rect.y + rect.height.saturating_sub(thickness) / 2;
            painter.fill_rect(
                Region::new(rect.x, y, rect.width, thickness.min(rect.height)),
                theme.tones.border,
            );
        }
        DividerOrientation::Vertical => {
            let x = rect.x + rect.width.saturating_sub(thickness) / 2;
            painter.fill_rect(
                Region::new(x, rect.y, thickness.min(rect.width), rect.height),
                theme.tones.border,
            );
        }
    }
}
