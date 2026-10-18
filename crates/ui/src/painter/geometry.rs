//! Shared pixel geometry helpers for the painter.

use common::Region;

use crate::luma::Luma;
use crate::shape::ScreenShape;
use crate::surface::Surface;

pub(super) fn fill_shaped_rows(
    surface: &mut dyn Surface,
    shape: ScreenShape,
    region: Region,
    rows: core::ops::Range<u16>,
    color: Luma,
) {
    let width = surface.width();
    let height = surface.height();
    for y in rows {
        let (left, right) = shape.row_span(y, width, height);
        let start = region.x.max(left);
        let end = region.right().min(right);
        if start < end {
            surface.fill_rect(Region::new(start, y, end - start, 1), color);
        }
    }
}

pub(super) fn offset_axis(center: u16, offset: u16, is_positive: bool) -> u16 {
    if is_positive {
        center.saturating_add(offset)
    } else {
        center.saturating_sub(offset)
    }
}

pub(super) fn inside_rounded(region: Region, radius: u16, x: u16, y: u16) -> bool {
    let left = i32::from(region.x);
    let top = i32::from(region.y);
    let right = i32::from(region.right());
    let bottom = i32::from(region.bottom());
    let radius = i32::from(radius);
    let x = i32::from(x);
    let y = i32::from(y);
    let center_x = if x < left + radius {
        left + radius
    } else if x >= right - radius {
        right - radius
    } else {
        return true;
    };
    let center_y = if y < top + radius {
        top + radius
    } else if y >= bottom - radius {
        bottom - radius
    } else {
        return true;
    };
    let dx = (x * 2 + 1) - center_x * 2;
    let dy = (y * 2 + 1) - center_y * 2;
    dx * dx + dy * dy <= (radius * 2) * (radius * 2)
}
