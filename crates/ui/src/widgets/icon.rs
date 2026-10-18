//! Standalone icons.

use common::Region;

use crate::assets::IconId;
use crate::layout::Size;
use crate::painter::Painter;
use crate::theme::Theme;

use super::paint::{colors, draw_icon_centered};
use super::{Widget, WidgetKind};

/// A theme icon size step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IconSize {
    /// Small icon.
    Sm,
    /// Default icon.
    Md,
    /// Large icon.
    Lg,
    /// Extra-large icon.
    Xl,
}

impl IconSize {
    /// Resolves the step against the active theme.
    pub fn pixels(self, theme: &Theme) -> u16 {
        match self {
            Self::Sm => theme.icons.sm,
            Self::Md => theme.icons.md,
            Self::Lg => theme.icons.lg,
            Self::Xl => theme.icons.xl,
        }
    }
}

/// Props for a standalone icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IconProps {
    /// Icon to draw.
    pub icon: IconId,
    /// Theme size step.
    pub size: IconSize,
}

/// Measures an icon.
pub(crate) fn measure(
    props: &IconProps,
    theme: &Theme,
    _cache: &mut crate::text::TextCache,
    _available: u16,
) -> Size {
    let size = props.size.pixels(theme);
    Size::new(size, size)
}

/// Paints an icon.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Icon(props) = &widget.kind else {
        return;
    };
    let colors = colors(theme, widget.state);
    let size = props.size.pixels(theme).min(rect.width).min(rect.height);
    draw_icon_centered(painter, props.icon, size, rect, colors.content);
}
