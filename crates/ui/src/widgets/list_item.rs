//! List rows.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::Size;
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, frame};
use super::{Widget, WidgetKind};

/// Props for one list row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ListItemProps<'a> {
    /// Primary row label.
    pub title: &'a str,
    /// Optional secondary label.
    pub subtitle: &'a str,
    /// Optional leading icon.
    pub leading: Option<IconId>,
    /// Optional trailing icon.
    pub trailing: Option<IconId>,
}

/// Measures a list row.
pub(crate) fn measure(
    props: &ListItemProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let title = measure_text(props.title, &theme.type_scale.body, 0, cache);
    let subtitle = measure_text(props.subtitle, &theme.type_scale.caption, 0, cache);
    let text_height = title.height.saturating_add(if props.subtitle.is_empty() {
        0
    } else {
        theme.spacing.xs.saturating_add(subtitle.height)
    });
    let inset = content_inset(theme);
    let width = if available == crate::layout::UNBOUNDED {
        240
    } else {
        available
    };
    Size::new(
        width,
        theme.min_touch_px.max(text_height.saturating_add(inset)),
    )
}

fn content_inset(theme: &Theme) -> u16 {
    theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.md)
        .saturating_mul(2)
}

/// Paints a list row.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::ListItem(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, 0);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.md);
    let mut text_region = region;
    if let Some(icon) = props.leading {
        let size = theme.icons.md;
        let x = region.x;
        let y = region.y + centered_offset(region.height, size);
        draw_icon(painter, icon, size, (x, y), colors.content);
        text_region.x = text_region.x.saturating_add(size + theme.spacing.md);
        text_region.width = text_region.width.saturating_sub(size + theme.spacing.md);
    }
    if let Some(icon) = props.trailing {
        let size = theme.icons.sm;
        let x = text_region.right().saturating_sub(size);
        let y = region.y + centered_offset(region.height, size);
        draw_icon(painter, icon, size, (x, y), colors.content);
        text_region.width = text_region.width.saturating_sub(size + theme.spacing.sm);
    }
    paint_text(painter, props, text_region, theme, colors.content);
    if !widget.state.is_selected() && !widget.state.is_pressed() {
        painter.fill_rect(
            Region::new(rect.x, rect.bottom().saturating_sub(1), rect.width, 1),
            theme.tones.border,
        );
    }
}

fn paint_text(
    painter: &mut Painter<'_>,
    props: &ListItemProps<'_>,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let title = measure_uncached(props.title, &theme.type_scale.body, region.width);
    let subtitle = measure_uncached(props.subtitle, &theme.type_scale.caption, region.width);
    let gap = if props.subtitle.is_empty() {
        0
    } else {
        theme.spacing.xs
    };
    let total = title
        .height
        .saturating_add(gap)
        .saturating_add(subtitle.height);
    let top = region.y + centered_offset(region.height, total);
    draw_text(
        painter,
        props.title,
        &theme.type_scale.body,
        (region.x, top),
        region.width,
        TextAlign::Start,
        color,
    );
    if !props.subtitle.is_empty() {
        draw_text(
            painter,
            props.subtitle,
            &theme.type_scale.caption,
            (region.x, top + title.height + gap),
            region.width,
            TextAlign::Start,
            color,
        );
    }
}
