//! Card surfaces for grouping content.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::luma::Luma;
use crate::painter::Painter;
use crate::text::TextCache;
use crate::theme::Theme;

use super::paint::shrink;
use super::{Widget, WidgetKind};

const UNBOUNDED_WIDTH: u16 = 240;

/// The tonal role of a card surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardTone {
    /// The default surface.
    Surface,
    /// A raised surface for overlays and emphasis.
    Raised,
}

impl CardTone {
    /// Resolves the tone against the active theme.
    pub fn color(self, theme: &Theme) -> Luma {
        match self {
            Self::Surface => theme.tones.surface,
            Self::Raised => theme.tones.surface_raised,
        }
    }
}

/// Props for a card surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CardProps {
    /// Surface tone.
    pub tone: CardTone,
}

/// Measures a card.
pub(crate) fn measure(
    _props: &CardProps,
    theme: &Theme,
    _cache: &mut TextCache,
    available: u16,
) -> Size {
    Size::new(
        if available == UNBOUNDED {
            UNBOUNDED_WIDTH
        } else {
            available
        },
        theme.min_touch_px,
    )
}

/// Paints a card surface.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Card(props) = &widget.kind else {
        return;
    };
    let radius = theme.radii.lg;
    painter.fill_rounded_rect(rect, radius, props.tone.color(theme));
    let border = theme.borders.thin.max(1);
    let inner = shrink(rect, border);
    if !inner.is_empty() {
        painter.stroke_rect(inner, border, theme.tones.border);
    }
}
