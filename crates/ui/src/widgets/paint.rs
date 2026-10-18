//! Widget painting and shared visual primitives.

use common::Region;

use crate::icon::draw_icon;
use crate::layout::LayoutEntry;
use crate::luma::Luma;
use crate::painter::Painter;
use crate::text::{draw_text, measure_uncached, TextAlign};
use crate::theme::{TextStyle, Theme};

use super::arena::WidgetArena;
use super::state::WidgetState;
use super::{Widget, WidgetKind};

use super::{
    button, checkbox, input, list_item, overlay, patterns, progress, radio, slider, status, tabs,
    toggle,
};

/// Paints one widget inside `rect`.
pub fn paint_widget(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    if rect.is_empty() {
        return;
    }
    match widget.kind {
        WidgetKind::None => {}
        WidgetKind::Button(_) | WidgetKind::IconButton(_) => {
            button::paint(painter, widget, rect, theme)
        }
        WidgetKind::ListItem(_) => list_item::paint(painter, widget, rect, theme),
        WidgetKind::Toggle(_) => toggle::paint(painter, widget, rect, theme),
        WidgetKind::Checkbox(_) => checkbox::paint(painter, widget, rect, theme),
        WidgetKind::Radio(_) => radio::paint(painter, widget, rect, theme),
        WidgetKind::Slider(_) => slider::paint(painter, widget, rect, theme),
        WidgetKind::Input(_) => input::paint(painter, widget, rect, theme),
        WidgetKind::Progress(_) | WidgetKind::Badge(_) => {
            progress::paint(painter, widget, rect, theme)
        }
        WidgetKind::Dialog(_) | WidgetKind::Toast(_) | WidgetKind::ContextMenu(_) => {
            overlay::paint(painter, widget, rect, theme)
        }
        WidgetKind::Tabs(_) => tabs::paint(painter, widget, rect, theme),
        WidgetKind::StatusIndicator(_) => status::paint(painter, widget, rect, theme),
        WidgetKind::ScreenScaffold(_)
        | WidgetKind::EmptyState(_)
        | WidgetKind::LoadingState(_)
        | WidgetKind::ErrorState(_)
        | WidgetKind::Confirm(_) => patterns::paint(painter, widget, rect, theme),
    }
}

/// Paints every widget in layout entry order, clipped to each entry.
pub fn paint_widgets(
    painter: &mut Painter<'_>,
    widgets: &WidgetArena<'_>,
    entries: &[LayoutEntry],
    theme: &Theme,
) {
    for entry in entries {
        let Some(widget) = widgets.find(entry.id) else {
            continue;
        };
        let rect = entry.rect.visible(entry.clip);
        if rect.is_empty() {
            continue;
        }
        if painter.push_clip(entry.clip) {
            paint_widget(painter, widget, rect, theme);
            painter.pop_clip();
        }
    }
}

/// Resolved colors for one widget state.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Colors {
    /// Surface fill.
    pub fill: Luma,
    /// Text and icon color.
    pub content: Luma,
    /// Border color.
    pub border: Luma,
    /// Border width.
    pub border_width: u16,
    /// Whether the widget is disabled.
    pub disabled: bool,
}

/// Resolves state precedence into concrete colors.
pub(crate) fn colors(theme: &Theme, state: WidgetState) -> Colors {
    let mut colors = Colors {
        fill: theme.tones.surface,
        content: theme.tones.text,
        border: theme.tones.border,
        border_width: theme.borders.thin,
        disabled: false,
    };
    if state.is_disabled() {
        colors.content = theme.tones.disabled;
        colors.border = theme.tones.disabled;
        colors.disabled = true;
        return colors;
    }
    if state.is_pressed() {
        colors.fill = theme.tones.pressed;
        colors.content = on(theme, colors.fill);
        colors.border = theme.tones.pressed;
    } else if state.is_selected() {
        colors.fill = theme.tones.selected;
        colors.content = on(theme, colors.fill);
        colors.border = theme.tones.selected;
    }
    if state.is_error() {
        colors.border = theme.tones.error;
        colors.border_width = theme.borders.thick;
    }
    if state.is_focused() {
        colors.border = theme.tones.focus;
        colors.border_width = theme.borders.thick + 1;
    }
    colors
}

/// A contrasting content color for a filled surface.
pub(crate) fn on(theme: &Theme, fill: Luma) -> Luma {
    if fill.is_lit() {
        Luma::BLACK
    } else {
        theme.tones.text
    }
}

/// Draws a widget frame, including focus, error, and disabled treatments.
pub(crate) fn frame(
    painter: &mut Painter<'_>,
    rect: Region,
    theme: &Theme,
    state: WidgetState,
    radius: u16,
) {
    let colors = colors(theme, state);
    if colors.disabled {
        painter.fill_rect(rect, colors.fill);
        dashed_rect(painter, rect, colors.border, theme.borders.thin);
        return;
    }
    let inner = shrink(rect, colors.border_width);
    if radius > 0 {
        painter.fill_rounded_rect(rect, radius, colors.border);
        painter.fill_rounded_rect(
            inner,
            radius.saturating_sub(colors.border_width),
            colors.fill,
        );
    } else {
        painter.fill_rect(rect, colors.border);
        painter.fill_rect(inner, colors.fill);
    }
}

/// The inner content rectangle shared by every framed widget.
pub(crate) fn content(rect: Region, theme: &Theme, padding: u16) -> Region {
    shrink(rect, theme.borders.thick + 1 + padding)
}

/// Shrinks a region uniformly, saturating at empty.
pub(crate) fn shrink(rect: Region, amount: u16) -> Region {
    let doubled = amount.saturating_mul(2);
    Region::new(
        rect.x.saturating_add(amount),
        rect.y.saturating_add(amount),
        rect.width.saturating_sub(doubled),
        rect.height.saturating_sub(doubled),
    )
}

/// Draws text horizontally centered and vertically centered in `region`.
pub(crate) fn draw_centered(
    painter: &mut Painter<'_>,
    text: &str,
    style: &TextStyle,
    region: Region,
    color: Luma,
) {
    let metrics = measure_uncached(text, style, region.width);
    let x = region.x;
    let y = centered_offset(region.height, metrics.height) + region.y;
    draw_text(
        painter,
        text,
        style,
        (x, y),
        region.width,
        TextAlign::Center,
        color,
    );
}

/// Draws start-aligned text vertically centered in `region`.
pub(crate) fn draw_start(
    painter: &mut Painter<'_>,
    text: &str,
    style: &TextStyle,
    region: Region,
    color: Luma,
) {
    let metrics = measure_uncached(text, style, region.width);
    let y = centered_offset(region.height, metrics.height) + region.y;
    draw_text(
        painter,
        text,
        style,
        (region.x, y),
        region.width,
        TextAlign::Start,
        color,
    );
}

/// Draws a square icon centered in `region`.
pub(crate) fn draw_icon_centered(
    painter: &mut Painter<'_>,
    icon: crate::assets::IconId,
    size: u16,
    region: Region,
    color: Luma,
) {
    let x = region.x + centered_offset(region.width, size);
    let y = region.y + centered_offset(region.height, size);
    draw_icon(painter, icon, size, (x, y), color);
}

/// A centered offset for a child inside available space.
pub(crate) fn centered_offset(available: u16, size: u16) -> u16 {
    available.saturating_sub(size) / 2
}

/// Draws a focus ring around a widget that has no frame of its own.
pub(crate) fn focus_ring(
    painter: &mut Painter<'_>,
    rect: Region,
    theme: &Theme,
    state: WidgetState,
) {
    if !state.is_focused() || state.is_disabled() {
        return;
    }
    let ring = shrink(rect, 1);
    if !ring.is_empty() {
        painter.stroke_rect(ring, theme.borders.thick + 1, theme.tones.focus);
    }
}

fn dashed_rect(painter: &mut Painter<'_>, rect: Region, color: Luma, thickness: u16) {
    let dash = 4u16;
    let gap = 4u16;
    let thickness = thickness.max(1).min(rect.width.min(rect.height));
    if thickness == 0 {
        return;
    }
    let mut x = rect.x;
    while x < rect.right() {
        let width = dash.min(rect.right() - x);
        painter.fill_rect(Region::new(x, rect.y, width, thickness), color);
        painter.fill_rect(
            Region::new(x, rect.bottom() - thickness, width, thickness),
            color,
        );
        x = x.saturating_add(dash + gap);
    }
    let mut y = rect.y;
    while y < rect.bottom() {
        let height = dash.min(rect.bottom() - y);
        painter.fill_rect(Region::new(rect.x, y, thickness, height), color);
        painter.fill_rect(
            Region::new(rect.right() - thickness, y, thickness, height),
            color,
        );
        y = y.saturating_add(dash + gap);
    }
}
