//! Progress bars and badges.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{measure_text, TextCache};
use crate::theme::Theme;

use super::paint::{colors, draw_centered, draw_start, frame, shrink};
use super::{Widget, WidgetKind};

/// Props for a progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProgressProps<'a> {
    /// Value from 0 to 100.
    pub value: u8,
    /// Optional label above the bar.
    pub label: &'a str,
    /// Draw a moving-looking segment instead of a value.
    pub indeterminate: bool,
}

/// Props for a short status badge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BadgeProps<'a> {
    /// Badge text.
    pub label: &'a str,
}

/// Measures a progress bar or badge.
pub(crate) fn measure(
    kind: &WidgetKind<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    match kind {
        WidgetKind::Progress(props) => measure_progress(props, theme, cache, available),
        WidgetKind::Badge(props) => measure_badge(props, theme, cache),
        _ => Size::ZERO,
    }
}

fn measure_progress(
    props: &ProgressProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let label = measure_text(props.label, &theme.type_scale.caption, 0, cache);
    let width = if available == UNBOUNDED {
        240
    } else {
        available
    };
    let height = if props.label.is_empty() {
        8 + theme.spacing.md * 2
    } else {
        label
            .height
            .saturating_add(theme.spacing.xs)
            .saturating_add(8)
            .saturating_add(theme.spacing.md * 2)
    };
    Size::new(width, height)
}

fn measure_badge(props: &BadgeProps<'_>, theme: &Theme, cache: &mut TextCache) -> Size {
    let metrics = measure_text(props.label, &theme.type_scale.caption, 0, cache);
    Size::new(
        metrics
            .width
            .saturating_add(theme.spacing.sm.saturating_mul(2)),
        theme
            .type_scale
            .caption
            .line_height_px
            .max(24)
            .saturating_add(theme.spacing.xs.saturating_mul(2)),
    )
}

/// Paints a progress bar or badge.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    match &widget.kind {
        WidgetKind::Progress(props) => paint_progress(painter, widget, props, rect, theme),
        WidgetKind::Badge(props) => paint_badge(painter, widget, props, rect, theme),
        _ => {}
    }
}

fn paint_progress(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &ProgressProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    let colors = colors(theme, widget.state);
    let region = shrink(rect, theme.spacing.md);
    let track_height = 8;
    let label_height = if props.label.is_empty() {
        0
    } else {
        theme.type_scale.caption.line_height_px
    };
    if !props.label.is_empty() {
        draw_start(
            painter,
            props.label,
            &theme.type_scale.caption,
            Region::new(region.x, region.y, region.width, label_height),
            colors.content,
        );
    }
    let track = Region::new(
        region.x,
        region
            .y
            .saturating_add(label_height)
            .saturating_add(theme.spacing.xs),
        region.width,
        track_height.min(region.height),
    );
    paint_bar(painter, track, props, theme, colors.content);
}

fn paint_bar(
    painter: &mut Painter<'_>,
    track: Region,
    props: &ProgressProps<'_>,
    theme: &Theme,
    active: crate::luma::Luma,
) {
    let radius = track.height / 2;
    let border = theme.borders.thin.max(1);
    painter.fill_rounded_rect(track, radius, theme.tones.border);
    painter.fill_rounded_rect(
        shrink(track, border),
        radius.saturating_sub(border),
        theme.tones.background,
    );
    let (start, width) = if props.indeterminate {
        let start = (u32::from(track.width) * 30 / 100) as u16;
        let width = (u32::from(track.width) * 35 / 100) as u16;
        (start, width)
    } else {
        let width = (u32::from(track.width) * u32::from(props.value.min(100)) / 100) as u16;
        (0, width)
    };
    if width > 0 {
        let bar = Region::new(track.x.saturating_add(start), track.y, width, track.height);
        painter.fill_rounded_rect(bar, radius, active);
    }
}

fn paint_badge(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &BadgeProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    frame(
        painter,
        rect,
        theme,
        widget.state,
        theme.radii.pill.min(rect.height / 2),
    );
    let colors = colors(theme, widget.state);
    let region = shrink(rect, theme.borders.thick + 1 + theme.spacing.xs);
    draw_centered(
        painter,
        props.label,
        &theme.type_scale.caption,
        region,
        colors.content,
    );
}
