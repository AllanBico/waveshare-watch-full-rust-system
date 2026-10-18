//! Numeric steppers.

use core::fmt::Write;

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, draw_start, frame};
use super::{Widget, WidgetKind};

const UNBOUNDED_WIDTH: u16 = 240;

/// Props for a numeric stepper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StepperProps<'a> {
    /// Row label.
    pub label: &'a str,
    /// Current value.
    pub value: u16,
    /// Minimum value.
    pub min: u16,
    /// Maximum value.
    pub max: u16,
    /// Change per press.
    pub step: u16,
    /// Optional unit suffix.
    pub unit: &'a str,
}

/// The part of a stepper a tap landed on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepperZone {
    /// Decrease the value.
    Decrement,
    /// Increase the value.
    Increment,
}

impl StepperProps<'_> {
    /// Maps a tap offset inside the widget to a button zone.
    pub fn zone(&self, x: u16, width: u16) -> StepperZone {
        if x < width / 2 {
            StepperZone::Decrement
        } else {
            StepperZone::Increment
        }
    }
}

/// Measures a stepper.
pub(crate) fn measure(
    props: &StepperProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let label = measure_text(props.label, &theme.type_scale.body, 0, cache);
    let inset = theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.xs)
        .saturating_mul(2);
    Size::new(
        if available == UNBOUNDED {
            UNBOUNDED_WIDTH
        } else {
            available
        },
        theme.min_touch_px.max(label.height.saturating_add(inset)),
    )
}

/// Paints a stepper.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Stepper(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.xs);
    let mut buffer = NumberBuffer::new();
    let _ = write!(&mut buffer, "{}", props.value);
    let value = buffer.text();
    let value_width = value_width(value, theme);
    let trailing = control_width(props, theme, region.height, value_width);
    let label = label_region(region, trailing, theme);
    draw_start(
        painter,
        props.label,
        &theme.type_scale.body,
        label,
        colors.content,
    );
    let controls = Region::new(
        region.right().saturating_sub(trailing),
        region.y,
        trailing.min(region.width),
        region.height,
    );
    paint_buttons(painter, controls, theme, colors.content);
    let (value_region, unit_region) = value_regions(props, controls, theme, value_width);
    paint_value(painter, value, value_region, theme, colors.content);
    if !props.unit.is_empty() {
        paint_unit(
            painter,
            props.unit,
            unit_region,
            theme,
            theme.tones.text_muted,
        );
    }
}

fn value_width(value: &str, theme: &Theme) -> u16 {
    measure_uncached(value, &theme.type_scale.body, 0).width
}

fn control_width(
    props: &StepperProps<'_>,
    theme: &Theme,
    region_height: u16,
    value_width: u16,
) -> u16 {
    let button = theme.min_touch_px.min(region_height);
    let unit = measure_uncached(props.unit, &theme.type_scale.caption, 0);
    let gap = theme.spacing.xs;
    let suffix = if props.unit.is_empty() {
        0
    } else {
        gap.saturating_add(unit.width)
    };
    button
        .saturating_mul(2)
        .saturating_add(value_width)
        .saturating_add(suffix)
        .saturating_add(gap.saturating_mul(3))
}

fn label_region(mut region: Region, trailing: u16, theme: &Theme) -> Region {
    region.width = region.width.saturating_sub(trailing + theme.spacing.md);
    region
}

fn value_regions(
    props: &StepperProps<'_>,
    controls: Region,
    theme: &Theme,
    value_width: u16,
) -> (Region, Region) {
    let button = theme.min_touch_px.min(controls.height);
    let gap = theme.spacing.xs;
    let unit = measure_uncached(props.unit, &theme.type_scale.caption, 0);
    let total = value_width.saturating_add(if props.unit.is_empty() {
        0
    } else {
        gap.saturating_add(unit.width)
    });
    let available = controls.width.saturating_sub(button.saturating_mul(2));
    let x = controls
        .x
        .saturating_add(button)
        .saturating_add(gap)
        .saturating_add(available.saturating_sub(total) / 2);
    (
        Region::new(x, controls.y, value_width, controls.height),
        Region::new(
            x.saturating_add(value_width).saturating_add(gap),
            controls.y,
            unit.width,
            controls.height,
        ),
    )
}

fn paint_buttons(
    painter: &mut Painter<'_>,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let button = theme.min_touch_px.min(region.height);
    let minus = Region::new(region.x, region.y, button, button);
    let plus = Region::new(
        region.right().saturating_sub(button),
        region.y,
        button,
        button,
    );
    paint_icon(painter, IconId::Minus, minus, theme, color);
    paint_icon(painter, IconId::Plus, plus, theme, color);
}

fn paint_value(
    painter: &mut Painter<'_>,
    value: &str,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let style = theme.type_scale.body;
    let metrics = measure_uncached(value, &style, region.width);
    let y = region.y + centered_offset(region.height, metrics.height);
    draw_text(
        painter,
        value,
        &style,
        (region.x, y),
        region.width,
        TextAlign::Start,
        color,
    );
}

fn paint_unit(
    painter: &mut Painter<'_>,
    unit: &str,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let style = theme.type_scale.caption;
    let metrics = measure_uncached(unit, &style, region.width);
    let y = region.y + centered_offset(region.height, metrics.height);
    draw_text(
        painter,
        unit,
        &style,
        (region.x, y),
        region.width,
        TextAlign::Start,
        color,
    );
}

fn paint_icon(
    painter: &mut Painter<'_>,
    icon: IconId,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let size = theme.icons.sm.min(region.width).min(region.height);
    let x = region.x + centered_offset(region.width, size);
    let y = region.y + centered_offset(region.height, size);
    draw_icon(painter, icon, size, (x, y), color);
}

struct NumberBuffer {
    bytes: [u8; 5],
    len: usize,
}

impl NumberBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 5],
            len: 0,
        }
    }

    fn text(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

impl Write for NumberBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let available = self.bytes.len().saturating_sub(self.len);
        let count = value.len().min(available);
        self.bytes[self.len..self.len + count].copy_from_slice(&value.as_bytes()[..count]);
        self.len = self.len.saturating_add(count);
        Ok(())
    }
}
