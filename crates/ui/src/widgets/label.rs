//! Non-interactive text labels.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::{TextStyle, Theme};

use super::paint::{centered_offset, colors};
use super::{Widget, WidgetKind};

/// The typography role a label renders with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextRole {
    /// Hero text.
    Display,
    /// Screen title.
    Title,
    /// Primary body text.
    Body,
    /// Secondary text.
    Caption,
    /// Control label.
    Label,
    /// Tabular numerals.
    Numeric,
}

impl TextRole {
    /// Resolves the role against the active theme.
    pub const fn style(self, theme: &Theme) -> TextStyle {
        match self {
            Self::Display => theme.type_scale.display,
            Self::Title => theme.type_scale.title,
            Self::Body => theme.type_scale.body,
            Self::Caption => theme.type_scale.caption,
            Self::Label => theme.type_scale.label,
            Self::Numeric => theme.type_scale.numeric,
        }
    }
}

/// Props for a non-interactive label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelProps<'a> {
    /// Text to draw.
    pub text: &'a str,
    /// Typography role.
    pub role: TextRole,
    /// Horizontal alignment.
    pub align: TextAlign,
    /// Optional leading icon.
    pub icon: Option<IconId>,
}

/// Measures a label.
pub(crate) fn measure(
    props: &LabelProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let style = props.role.style(theme);
    let max_width = if available == UNBOUNDED { 0 } else { available };
    let text = measure_text(props.text, &style, max_width, cache);
    if props.icon.is_some() {
        let icon = theme.icons.sm;
        Size::new(
            text.width.saturating_add(icon + theme.spacing.sm),
            text.height.max(icon),
        )
    } else {
        Size::new(text.width, text.height)
    }
}

/// Paints a label.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Label(props) = &widget.kind else {
        return;
    };
    let color = colors(theme, widget.state).content;
    let style = props.role.style(theme);
    let mut text_region = rect;
    if let Some(icon) = props.icon {
        let size = theme.icons.sm.min(rect.height).min(rect.width);
        let y = rect.y + centered_offset(rect.height, size);
        draw_icon(painter, icon, size, (rect.x, y), color);
        let offset = size.saturating_add(theme.spacing.sm);
        text_region.x = rect.x.saturating_add(offset);
        text_region.width = rect.width.saturating_sub(offset);
    }
    let metrics = measure_uncached(props.text, &style, text_region.width);
    let y = text_region.y + centered_offset(text_region.height, metrics.height);
    draw_text(
        painter,
        props.text,
        &style,
        (text_region.x, y),
        text_region.width,
        props.align,
        color,
    );
}
