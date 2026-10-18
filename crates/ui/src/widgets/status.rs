//! Status indicator lines.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::Size;
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, shrink};
use super::{Widget, WidgetKind};

/// The semantic status of an indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// No activity.
    Idle,
    /// Healthy or complete.
    Ok,
    /// Needs attention.
    Warning,
    /// Failed.
    Error,
}

impl Status {
    /// The icon shown for this status.
    pub const fn icon(self) -> IconId {
        match self {
            Self::Idle => IconId::Info,
            Self::Ok => IconId::CheckCircle,
            Self::Warning => IconId::Warning,
            Self::Error => IconId::XCircle,
        }
    }
}

/// Props for a status indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusIndicatorProps<'a> {
    /// Status label.
    pub label: &'a str,
    /// Semantic status.
    pub status: Status,
}

/// Measures a status indicator.
pub(crate) fn measure(
    props: &StatusIndicatorProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    _available: u16,
) -> Size {
    let text = measure_text(props.label, &theme.type_scale.caption, 0, cache);
    let icon = theme.icons.sm.saturating_add(theme.spacing.sm);
    Size::new(
        text.width.saturating_add(icon),
        text.height.max(theme.icons.sm),
    )
}

/// Paints a status indicator.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::StatusIndicator(props) = &widget.kind else {
        return;
    };
    let colors = colors(theme, widget.state);
    let color = if widget.state.is_disabled() {
        colors.content
    } else {
        match props.status {
            Status::Error => theme.tones.error,
            Status::Warning | Status::Ok => colors.content,
            Status::Idle => theme.tones.text_muted,
        }
    };
    let region = shrink(rect, theme.borders.thin);
    let size = theme.icons.sm.min(region.height);
    let y = region.y + centered_offset(region.height, size);
    draw_icon(painter, props.status.icon(), size, (region.x, y), color);
    let text = Region::new(
        region.x.saturating_add(size + theme.spacing.sm),
        region.y,
        region.width.saturating_sub(size + theme.spacing.sm),
        region.height,
    );
    let style = theme.type_scale.caption;
    let metrics = measure_uncached(props.label, &style, text.width);
    let top = text.y + centered_offset(text.height, metrics.height);
    draw_text(
        painter,
        props.label,
        &style,
        (text.x, top),
        text.width,
        TextAlign::Start,
        color,
    );
}
