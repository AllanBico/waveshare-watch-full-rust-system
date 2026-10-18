//! Circular progress rings.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::TextCache;
use crate::theme::Theme;

use super::paint::{centered_offset, colors, draw_centered, draw_dot, shrink};
use super::{Widget, WidgetKind};

const STEPS: u16 = 64;
const QUARTER_STEPS: u16 = 16;
const UNIT: i32 = 1024;
const MIN_PX: u16 = 120;
const MAX_PX: u16 = 180;
const TRACK_RADIUS: u16 = 1;

const QUARTER: [(i16, i16); 17] = [
    (1024, 0),
    (1019, 100),
    (1004, 200),
    (980, 297),
    (946, 392),
    (903, 483),
    (851, 569),
    (792, 650),
    (724, 724),
    (650, 792),
    (569, 851),
    (483, 903),
    (392, 946),
    (297, 980),
    (200, 1004),
    (100, 1019),
    (0, 1024),
];

/// Props for a circular progress ring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RingProps<'a> {
    /// Value from 0 to 100.
    pub value: u8,
    /// Label below the value.
    pub label: &'a str,
}

/// Measures a ring.
pub(crate) fn measure(
    _props: &RingProps<'_>,
    _theme: &Theme,
    _cache: &mut TextCache,
    available: u16,
) -> Size {
    let side = if available == UNBOUNDED {
        MIN_PX
    } else {
        available.clamp(MIN_PX, MAX_PX)
    };
    Size::new(side, side)
}

/// Paints a ring.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Ring(props) = &widget.kind else {
        return;
    };
    let colors = colors(theme, widget.state);
    let side = rect.width.min(rect.height);
    if side == 0 {
        return;
    }
    let thickness = theme.spacing.sm.max(1);
    let radius = side / 2;
    let ring_radius = radius.saturating_sub(thickness);
    let center = (rect.x + radius, rect.y + radius);
    let progress = u32::from(props.value.min(100)) * u32::from(STEPS) / 100;
    draw_ring(
        painter,
        center,
        ring_radius,
        STEPS,
        TRACK_RADIUS,
        theme.tones.border,
    );
    draw_ring(
        painter,
        center,
        ring_radius,
        progress as u16,
        thickness / 2,
        colors.content,
    );
    paint_center(painter, props, rect, theme, colors.content);
}

fn draw_ring(
    painter: &mut Painter<'_>,
    center: (u16, u16),
    radius: u16,
    steps: u16,
    dot_radius: u16,
    color: crate::luma::Luma,
) {
    for step in 0..steps {
        let (unit_x, unit_y) = unit_vector(step);
        let x = i32::from(center.0) + unit_x * i32::from(radius) / UNIT;
        let y = i32::from(center.1) + unit_y * i32::from(radius) / UNIT;
        draw_dot(
            painter,
            (x.max(0) as u16, y.max(0) as u16),
            dot_radius,
            color,
        );
    }
}

fn unit_vector(step: u16) -> (i32, i32) {
    let index = step % STEPS;
    let quadrant = index / QUARTER_STEPS;
    let within = index % QUARTER_STEPS;
    let (cosine, sine) = QUARTER[usize::from(within)];
    let (cosine, sine) = (i32::from(cosine), i32::from(sine));
    match quadrant {
        0 => (sine, -cosine),
        1 => (cosine, sine),
        2 => (-sine, cosine),
        _ => (-cosine, -sine),
    }
}

fn paint_center(
    painter: &mut Painter<'_>,
    props: &RingProps<'_>,
    rect: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let thickness = theme.spacing.sm.max(1);
    let inner = shrink(rect, thickness.saturating_mul(2));
    let value_height = theme.type_scale.title.line_height_px;
    let label_height = theme.type_scale.caption.line_height_px;
    let gap = theme.spacing.xs;
    let total = value_height
        .saturating_add(gap)
        .saturating_add(label_height);
    let top = inner.y + centered_offset(inner.height, total);
    let buffer = PercentBuffer::new(props.value);
    draw_centered(
        painter,
        buffer.text(),
        &theme.type_scale.title,
        Region::new(inner.x, top, inner.width, value_height),
        color,
    );
    draw_centered(
        painter,
        props.label,
        &theme.type_scale.caption,
        Region::new(
            inner.x,
            top.saturating_add(value_height).saturating_add(gap),
            inner.width,
            label_height,
        ),
        theme.tones.text_muted,
    );
}

struct PercentBuffer {
    bytes: [u8; 4],
    len: usize,
}

impl PercentBuffer {
    fn new(value: u8) -> Self {
        let value = value.min(100);
        let mut bytes = [b'%'; 4];
        let len = if value == 100 {
            bytes[..3].copy_from_slice(b"100");
            4
        } else {
            bytes[0] = b'0' + value / 10;
            bytes[1] = b'0' + value % 10;
            3
        };
        Self { bytes, len }
    }

    fn text(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}
