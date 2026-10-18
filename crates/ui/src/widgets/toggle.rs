//! Toggle switches.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{measure_text, TextCache};
use crate::theme::Theme;

use super::paint::{colors, content, draw_start, frame, shrink};
use super::{Widget, WidgetKind};

/// Switch track width in pixels.
pub(crate) const TRACK_WIDTH: u16 = 56;
/// Switch track height in pixels.
pub(crate) const TRACK_HEIGHT: u16 = 32;
const KNOB_INSET: u16 = 4;

/// Props for an on/off switch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToggleProps<'a> {
    /// Whether the switch is on.
    pub checked: bool,
    /// Row label.
    pub label: &'a str,
}

/// Measures a toggle row.
pub(crate) fn measure(
    props: &ToggleProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let label = measure_text(props.label, &theme.type_scale.body, 0, cache);
    let inset = inset(theme);
    let width = if available == UNBOUNDED {
        240
    } else {
        available
    };
    Size::new(
        width,
        theme.min_touch_px.max(label.height.saturating_add(inset)),
    )
}

fn inset(theme: &Theme) -> u16 {
    theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.md)
        .saturating_mul(2)
}

/// Paints a toggle row.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Toggle(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.md);
    let track = Region::new(
        region.right().saturating_sub(TRACK_WIDTH),
        region.y + (region.height.saturating_sub(TRACK_HEIGHT)) / 2,
        TRACK_WIDTH.min(region.width),
        TRACK_HEIGHT.min(region.height),
    );
    let mut label_region = region;
    label_region.width = label_region
        .width
        .saturating_sub(TRACK_WIDTH + theme.spacing.md);
    draw_start(
        painter,
        props.label,
        &theme.type_scale.body,
        label_region,
        colors.content,
    );
    paint_switch(
        painter,
        track,
        props.checked,
        theme,
        colors.content,
        colors.fill,
    );
}

pub(crate) fn paint_switch(
    painter: &mut Painter<'_>,
    track: Region,
    checked: bool,
    theme: &Theme,
    active: crate::luma::Luma,
    inactive: crate::luma::Luma,
) {
    let radius = track.height / 2;
    let border = theme.borders.thick.max(1);
    if checked {
        painter.fill_rounded_rect(track, radius, active);
    } else {
        painter.fill_rounded_rect(track, radius, theme.tones.border);
        painter.fill_rounded_rect(
            shrink(track, border),
            radius.saturating_sub(border),
            inactive,
        );
    }
    let knob_size = track.height.saturating_sub(KNOB_INSET.saturating_mul(2));
    let knob_radius = knob_size / 2;
    let knob_color = if checked {
        super::paint::on(theme, active)
    } else {
        active
    };
    let x = if checked {
        track.right().saturating_sub(knob_size + KNOB_INSET)
    } else {
        track.x.saturating_add(KNOB_INSET)
    };
    let knob = Region::new(x, track.y + KNOB_INSET, knob_size, knob_size);
    painter.fill_rounded_rect(knob, knob_radius, knob_color);
}
