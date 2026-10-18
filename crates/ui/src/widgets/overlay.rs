//! Dialogs, toasts, and context menus.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, draw_start, frame, shrink};
use super::{Widget, WidgetKind};

/// Props for a modal message body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DialogProps<'a> {
    /// Dialog heading.
    pub title: &'a str,
    /// Dialog body text.
    pub body: &'a str,
}

/// Props for a transient message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToastProps<'a> {
    /// Message text.
    pub message: &'a str,
    /// Optional leading icon.
    pub icon: Option<IconId>,
}

/// Props for a menu of actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ContextMenuProps<'a> {
    /// Menu items in paint order.
    pub items: &'a [MenuItem<'a>],
}

/// One context-menu action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MenuItem<'a> {
    /// Action label.
    pub label: &'a str,
    /// Optional leading icon.
    pub icon: Option<IconId>,
}

impl MenuItem<'_> {
    /// An empty item placeholder.
    pub const EMPTY: Self = Self {
        label: "",
        icon: None,
    };
}

const MENU_ROW_HEIGHT: u16 = 52;

/// Measures a dialog, toast, or context menu.
pub(crate) fn measure(
    kind: &WidgetKind<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    match kind {
        WidgetKind::Dialog(props) => measure_dialog(props, theme, cache, available),
        WidgetKind::Toast(props) => measure_toast(props, theme, cache),
        WidgetKind::ContextMenu(props) => measure_menu(props, theme, cache, available),
        _ => Size::ZERO,
    }
}

fn measure_dialog(
    props: &DialogProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let title = measure_text(props.title, &theme.type_scale.title, 0, cache);
    let body = measure_text(props.body, &theme.type_scale.body, 0, cache);
    let inset = theme.spacing.lg.saturating_mul(2);
    let width = if available == UNBOUNDED {
        300
    } else {
        available
    };
    let height = title
        .height
        .saturating_add(theme.spacing.md)
        .saturating_add(body.height)
        .saturating_add(inset);
    Size::new(width, height)
}

fn measure_toast(props: &ToastProps<'_>, theme: &Theme, cache: &mut TextCache) -> Size {
    let text = measure_text(props.message, &theme.type_scale.caption, 0, cache);
    let icon = if props.icon.is_some() {
        theme.icons.sm.saturating_add(theme.spacing.sm)
    } else {
        0
    };
    let inset = theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.md)
        .saturating_mul(2);
    Size::new(
        text.width
            .saturating_add(icon)
            .saturating_add(inset)
            .max(theme.min_touch_px),
        text.height.saturating_add(inset).max(48),
    )
}

fn measure_menu(
    props: &ContextMenuProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let mut width = 0u16;
    for item in props.items {
        let text = measure_text(item.label, &theme.type_scale.caption, 0, cache);
        let icon = if item.icon.is_some() {
            theme.icons.sm.saturating_add(theme.spacing.sm)
        } else {
            0
        };
        width = width.max(text.width.saturating_add(icon));
    }
    let inset = theme.spacing.md.saturating_mul(2);
    let width = width.saturating_add(inset);
    let width = if available == UNBOUNDED {
        width.max(160)
    } else {
        available.min(width.max(160))
    };
    let height = (props.items.len() as u16)
        .saturating_mul(MENU_ROW_HEIGHT)
        .saturating_add(theme.spacing.sm.saturating_mul(2));
    Size::new(width, height)
}

/// Paints a dialog, toast, or context menu.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    match &widget.kind {
        WidgetKind::Dialog(props) => paint_dialog(painter, widget, props, rect, theme),
        WidgetKind::Toast(props) => paint_toast(painter, widget, props, rect, theme),
        WidgetKind::ContextMenu(props) => paint_menu(painter, widget, props, rect, theme),
        _ => {}
    }
}

fn raised_frame(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    rect: Region,
    theme: &Theme,
    radius: u16,
) {
    frame(painter, rect, theme, widget.state, radius);
    let border = colors(theme, widget.state).border_width;
    painter.fill_rect(shrink(rect, border), theme.tones.surface_raised);
}

fn paint_dialog(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &DialogProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    raised_frame(painter, widget, rect, theme, theme.radii.lg);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.lg);
    let title = measure_uncached(props.title, &theme.type_scale.title, region.width);
    draw_text(
        painter,
        props.title,
        &theme.type_scale.title,
        (region.x, region.y),
        region.width,
        TextAlign::Start,
        colors.content,
    );
    let divider_y = region
        .y
        .saturating_add(title.height)
        .saturating_add(theme.spacing.sm);
    painter.fill_rect(
        Region::new(region.x, divider_y, region.width, theme.borders.thin.max(1)),
        theme.tones.border,
    );
    draw_text(
        painter,
        props.body,
        &theme.type_scale.body,
        (region.x, divider_y.saturating_add(theme.spacing.md)),
        region.width,
        TextAlign::Start,
        colors.content,
    );
}

fn paint_toast(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &ToastProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    let radius = theme.radii.pill.min(rect.height / 2);
    raised_frame(painter, widget, rect, theme, radius);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.md);
    let mut text_region = region;
    if let Some(icon) = props.icon {
        let size = theme.icons.sm;
        let y = region.y + centered_offset(region.height, size);
        draw_icon(painter, icon, size, (region.x, y), colors.content);
        text_region.x = text_region.x.saturating_add(size + theme.spacing.sm);
        text_region.width = text_region.width.saturating_sub(size + theme.spacing.sm);
    }
    draw_start(
        painter,
        props.message,
        &theme.type_scale.caption,
        text_region,
        colors.content,
    );
}

fn paint_menu(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &ContextMenuProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    raised_frame(painter, widget, rect, theme, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = shrink(rect, theme.spacing.sm);
    for (index, item) in props.items.iter().enumerate() {
        let top = region
            .y
            .saturating_add((index as u16).saturating_mul(MENU_ROW_HEIGHT));
        let row = Region::new(
            region.x,
            top,
            region.width,
            MENU_ROW_HEIGHT.min(region.bottom().saturating_sub(top)),
        );
        paint_menu_row(painter, item, row, theme, colors.content);
    }
}

fn paint_menu_row(
    painter: &mut Painter<'_>,
    item: &MenuItem<'_>,
    row: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    if row.is_empty() {
        return;
    }
    let content = shrink(row, theme.spacing.sm);
    let mut text = content;
    if let Some(icon) = item.icon {
        let size = theme.icons.sm;
        let y = content.y + centered_offset(content.height, size);
        draw_icon(painter, icon, size, (content.x, y), color);
        text.x = text.x.saturating_add(size + theme.spacing.sm);
        text.width = text.width.saturating_sub(size + theme.spacing.sm);
    }
    draw_start(painter, item.label, &theme.type_scale.caption, text, color);
}
