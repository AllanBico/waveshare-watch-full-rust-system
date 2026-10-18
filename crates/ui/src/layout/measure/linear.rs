//! Row and column measurement.

use crate::text::TextCache;

use super::super::constraint::{Constraints, Size, UNBOUNDED};
use super::super::fill::{gap_total, FillCursor};
use super::super::insets::Axis;
use super::super::node::Node;
use super::super::LayoutError;
use super::measure;

pub(super) fn measure_row(
    children: &[Node<'_>],
    spacing: u16,
    max: Size,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    let available = max.width;
    let mut fixed = 0u32;
    let mut fill_weight = 0u32;
    let mut height = 0u16;
    for child in children {
        if available != UNBOUNDED && child.width.is_fill() {
            fill_weight = fill_weight.saturating_add(u32::from(child.width.weight()));
            continue;
        }
        let size = measure(child, Constraints::loose(max), cache, depth + 1)?;
        height = height.max(size.height);
        fixed = fixed.saturating_add(u32::from(size.width));
    }
    fixed = fixed.saturating_add(gap_total(children.len(), spacing));
    if available != UNBOUNDED && fill_weight > 0 {
        let pass = FillPass {
            children,
            available,
            fixed,
            weight: fill_weight,
            cross_max: max.height,
            axis: Axis::Horizontal,
        };
        height = height.max(pass.run(cache, depth)?);
        return Ok(Size::new(available, height));
    }
    Ok(Size::new(fixed.min(u32::from(u16::MAX)) as u16, height))
}

pub(super) fn measure_column(
    children: &[Node<'_>],
    spacing: u16,
    max: Size,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    let available = max.height;
    let mut fixed = 0u32;
    let mut fill_weight = 0u32;
    let mut width = 0u16;
    for child in children {
        if available != UNBOUNDED && child.height.is_fill() {
            fill_weight = fill_weight.saturating_add(u32::from(child.height.weight()));
            continue;
        }
        let size = measure(child, Constraints::loose(max), cache, depth + 1)?;
        width = width.max(size.width);
        fixed = fixed.saturating_add(u32::from(size.height));
    }
    fixed = fixed.saturating_add(gap_total(children.len(), spacing));
    if available != UNBOUNDED && fill_weight > 0 {
        let pass = FillPass {
            children,
            available,
            fixed,
            weight: fill_weight,
            cross_max: max.width,
            axis: Axis::Vertical,
        };
        width = width.max(pass.run(cache, depth)?);
        return Ok(Size::new(width, available));
    }
    Ok(Size::new(width, fixed.min(u32::from(u16::MAX)) as u16))
}

struct FillPass<'a> {
    children: &'a [Node<'a>],
    available: u16,
    fixed: u32,
    weight: u32,
    cross_max: u16,
    axis: Axis,
}

impl FillPass<'_> {
    fn run(&self, cache: &mut TextCache, depth: u8) -> Result<u16, LayoutError> {
        let used = self.fixed.min(u32::from(self.available)) as u16;
        let mut cursor = FillCursor::new(self.available.saturating_sub(used), self.weight);
        let mut cross = 0u16;
        for child in self.children {
            let length = match self.axis {
                Axis::Horizontal => child.width,
                Axis::Vertical => child.height,
            };
            if !length.is_fill() {
                continue;
            }
            let share = cursor.take(length.weight());
            let constraints = match self.axis {
                Axis::Horizontal => Size::new(share, self.cross_max),
                Axis::Vertical => Size::new(self.cross_max, share),
            };
            let size = measure(child, Constraints::loose(constraints), cache, depth + 1)?;
            cross = cross.max(match self.axis {
                Axis::Horizontal => size.height,
                Axis::Vertical => size.width,
            });
        }
        Ok(cross)
    }
}
