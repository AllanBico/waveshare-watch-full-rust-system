//! Rounded-rectangle geometry for the calibration overlay.

use common::Region;
use ui::{Luma, Painter};

/// Insets a region on every edge.
pub(super) fn inset(region: Region, amount: u16) -> Region {
    let doubled = amount.saturating_mul(2);
    Region::new(
        region.x.saturating_add(amount),
        region.y.saturating_add(amount),
        region.width.saturating_sub(doubled),
        region.height.saturating_sub(doubled),
    )
}

/// Reduces every corner radius by `amount`.
pub(super) fn reduced(radii: [u16; 4], amount: u16) -> [u16; 4] {
    radii.map(|radius| radius.saturating_sub(amount))
}

/// Draws a one-pixel rounded-rectangle outline.
pub(super) fn stroke(painter: &mut Painter<'_>, region: Region, radii: [u16; 4], color: Luma) {
    if region.is_empty() {
        return;
    }
    let radii = clamp(region, radii);
    let right = region.right() - 1;
    let bottom = region.bottom() - 1;
    stroke_edges(painter, region, radii, right, bottom, color);
    stroke_arcs(painter, region, radii, right, bottom, color);
}

fn stroke_edges(
    painter: &mut Painter<'_>,
    region: Region,
    radii: [u16; 4],
    right: u16,
    bottom: u16,
    color: Luma,
) {
    let [top_left, top_right, bottom_right, bottom_left] = radii;
    let top_end = region.right().saturating_sub(top_right + 1);
    let bottom_end = region.right().saturating_sub(bottom_right + 1);
    let right_start = region.bottom().saturating_sub(bottom_right + 1);
    let left_end = region.bottom().saturating_sub(bottom_left + 1);
    painter.line(region.x + top_left, region.y, top_end, region.y, color);
    painter.line(right, region.y + top_right, right, right_start, color);
    painter.line(region.x + bottom_left, bottom, bottom_end, bottom, color);
    painter.line(region.x, region.y + top_left, region.x, left_end, color);
}

fn stroke_arcs(
    painter: &mut Painter<'_>,
    region: Region,
    radii: [u16; 4],
    right: u16,
    bottom: u16,
    color: Luma,
) {
    let [top_left, top_right, bottom_right, bottom_left] = radii;
    top_arcs(painter, region, top_left, top_right, right, color);
    bottom_arcs(
        painter,
        region,
        bottom_left,
        bottom_right,
        right,
        bottom,
        color,
    );
}

fn top_arcs(
    painter: &mut Painter<'_>,
    region: Region,
    top_left: u16,
    top_right: u16,
    right: u16,
    color: Luma,
) {
    arc(
        painter,
        (region.x + top_left, region.y + top_left),
        top_left,
        false,
        false,
        color,
    );
    arc(
        painter,
        (right - top_right, region.y + top_right),
        top_right,
        true,
        false,
        color,
    );
}

fn bottom_arcs(
    painter: &mut Painter<'_>,
    region: Region,
    bottom_left: u16,
    bottom_right: u16,
    right: u16,
    bottom: u16,
    color: Luma,
) {
    arc(
        painter,
        (right - bottom_right, bottom - bottom_right),
        bottom_right,
        true,
        true,
        color,
    );
    arc(
        painter,
        (region.x + bottom_left, bottom - bottom_left),
        bottom_left,
        false,
        true,
        color,
    );
}

/// Draws a rounded-rectangle outline with `thickness` pixels.
pub(super) fn stroke_thick(
    painter: &mut Painter<'_>,
    region: Region,
    radii: [u16; 4],
    thickness: u16,
    color: Luma,
) {
    let mut region = region;
    let mut radii = radii;
    for _ in 0..thickness {
        stroke(painter, region, radii, color);
        region = inset(region, 1);
        radii = reduced(radii, 1);
    }
}

/// Fills everything outside the rounded rectangle with `color`.
pub(super) fn mask_outside(
    painter: &mut Painter<'_>,
    region: Region,
    radii: [u16; 4],
    color: Luma,
) {
    if region.is_empty() {
        return;
    }
    for y in region.y..region.bottom() {
        let (left, right) = row_span(region, radii, y);
        if left > region.x {
            painter.fill_rect(Region::new(region.x, y, left - region.x, 1), color);
        }
        if right < region.right() {
            painter.fill_rect(Region::new(right, y, region.right() - right, 1), color);
        }
    }
}

fn row_span(region: Region, radii: [u16; 4], y: u16) -> (u16, u16) {
    let [top_left, top_right, bottom_right, bottom_left] = clamp(region, radii);
    let bottom_edge = region.bottom() - 1;
    let mut left = region.x;
    let mut right = region.right();

    if y < region.y + top_left {
        let dy = region.y + top_left - y;
        left = (region.x + top_left).saturating_sub(arc_dx(top_left, dy));
    } else if y > bottom_edge.saturating_sub(bottom_left) {
        let dy = y - (bottom_edge - bottom_left);
        left = (region.x + bottom_left).saturating_sub(arc_dx(bottom_left, dy));
    }

    if y < region.y + top_right {
        let dy = region.y + top_right - y;
        right = region.right() - 1 - top_right + arc_dx(top_right, dy) + 1;
    } else if y > bottom_edge.saturating_sub(bottom_right) {
        let dy = y - (bottom_edge - bottom_right);
        right = region.right() - 1 - bottom_right + arc_dx(bottom_right, dy) + 1;
    }

    let left = left.min(region.right());
    let right = right.min(region.right()).max(left);
    (left, right)
}

fn clamp(region: Region, radii: [u16; 4]) -> [u16; 4] {
    let limit = region.width.min(region.height) / 2;
    radii.map(|radius| radius.min(limit))
}

fn arc(
    painter: &mut Painter<'_>,
    center: (u16, u16),
    radius: u16,
    is_right: bool,
    is_bottom: bool,
    color: Luma,
) {
    if radius == 0 {
        return;
    }
    let (center_x, center_y) = center;
    let squared = u32::from(radius) * u32::from(radius);
    for offset in 0..=u32::from(radius) {
        let other = isqrt(squared - offset * offset);
        let x = axis(center_x, offset as u16, is_right);
        let y = axis(center_y, other as u16, is_bottom);
        painter.set_pixel(x, y, color);
        let x = axis(center_x, other as u16, is_right);
        let y = axis(center_y, offset as u16, is_bottom);
        painter.set_pixel(x, y, color);
    }
}

fn axis(center: u16, offset: u16, is_positive: bool) -> u16 {
    if is_positive {
        center.saturating_add(offset)
    } else {
        center.saturating_sub(offset)
    }
}

fn arc_dx(radius: u16, dy: u16) -> u16 {
    let squared = u32::from(radius) * u32::from(radius);
    let offset = u32::from(dy.min(radius));
    isqrt(squared - offset * offset) as u16
}

fn isqrt(value: u32) -> u32 {
    let mut result = 0u32;
    let mut bit = 1u32 << 30;
    let mut remainder = value;
    while bit > remainder {
        bit >>= 2;
    }
    while bit != 0 {
        if remainder >= result + bit {
            remainder -= result + bit;
            result = (result >> 1) + bit;
        } else {
            result >>= 1;
        }
        bit >>= 2;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const REGION: Region = Region::new(0, 0, 100, 100);
    const RADIUS: [u16; 4] = [20, 20, 20, 20];

    #[test]
    fn corner_rows_are_inset_by_the_arc() {
        assert_eq!(row_span(REGION, RADIUS, 0), (20, 80));
        assert_eq!(row_span(REGION, RADIUS, 99), (20, 80));
    }

    #[test]
    fn straight_rows_use_the_full_width() {
        assert_eq!(row_span(REGION, RADIUS, 50), (0, 100));
    }

    #[test]
    fn zero_radius_is_a_plain_rectangle() {
        assert_eq!(row_span(REGION, [0; 4], 0), (0, 100));
        assert_eq!(row_span(REGION, [0; 4], 99), (0, 100));
    }

    #[test]
    fn integer_square_root_floors() {
        assert_eq!(isqrt(0), 0);
        assert_eq!(isqrt(1), 1);
        assert_eq!(isqrt(15), 3);
        assert_eq!(isqrt(16), 4);
        assert_eq!(isqrt(99), 9);
    }
}
