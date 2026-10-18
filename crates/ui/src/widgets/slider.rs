//! Sliders.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{measure_text, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, draw_start, frame, shrink};
use super::{Widget, WidgetKind};

/// Props for a continuous value slider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SliderProps<'a> {
    /// Value from 0 to 100.
    pub value: u8,
    /// Optional label above the track.
    pub label: &'a str,
}

/// Measures a slider row.
pub(crate) fn measure(
    props: &SliderProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let label = measure_text(props.label, &theme.type_scale.caption, 0, cache);
    let inset = theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.md)
        .saturating_mul(2);
    let width = if available == UNBOUNDED {
        240
    } else {
        available
    };
    let height = theme.min_touch_px.max(label.height.saturating_add(inset));
    Size::new(width, height)
}

/// Paints a slider row.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Slider(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.md);
    let (label_region, track) = track_regions(region, props.label.is_empty(), theme);
    if let Some(label_region) = label_region {
        draw_start(
            painter,
            props.label,
            &theme.type_scale.caption,
            label_region,
            colors.content,
        );
    }
    paint_track(painter, track, props.value, theme, colors.content);
}

fn track_regions(region: Region, is_label_empty: bool, theme: &Theme) -> (Option<Region>, Region) {
    let thumb = 24;
    let track_height = 8;
    let label_height = if is_label_empty {
        0
    } else {
        theme.type_scale.caption.line_height_px
    };
    let gap = if is_label_empty { 0 } else { theme.spacing.xs };
    let total = label_height
        .saturating_add(gap)
        .saturating_add(thumb)
        .min(region.height);
    let top = region.y + centered_offset(region.height, total);
    let label = if is_label_empty {
        None
    } else {
        Some(Region::new(region.x, top, region.width, label_height))
    };
    let track_top = top + label_height + gap + (thumb.saturating_sub(track_height)) / 2;
    let track = Region::new(
        region.x,
        track_top,
        region.width,
        track_height.min(region.height),
    );
    (label, track)
}

fn paint_track(
    painter: &mut Painter<'_>,
    track: Region,
    value: u8,
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
    let percent = u32::from(value.min(100));
    let filled = ((u32::from(track.width) * percent) / 100) as u16;
    if filled > 0 {
        let filled_region = Region::new(track.x, track.y, filled, track.height);
        painter.fill_rounded_rect(filled_region, radius, active);
    }
    let thumb = 24.min(track.height.saturating_add(16));
    let x = track
        .x
        .saturating_add(filled)
        .saturating_sub(thumb / 2)
        .min(track.right().saturating_sub(thumb));
    let y = track
        .y
        .saturating_sub((thumb.saturating_sub(track.height)) / 2);
    painter.fill_rounded_rect(Region::new(x, y, thumb, thumb), thumb / 2, active);
}
