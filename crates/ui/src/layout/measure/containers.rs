//! Stack, grid, list, and scroll measurement.

use crate::text::TextCache;

use super::super::constraint::{Constraints, Size, UNBOUNDED};
use super::super::grid::GridSpec;
use super::super::insets::Axis;
use super::super::node::Node;
use super::super::LayoutError;
use super::measure;

pub(super) fn measure_stack(
    children: &[Node<'_>],
    max: Size,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    let mut size = Size::ZERO;
    for child in children {
        let child_size = measure(child, Constraints::loose(max), cache, depth + 1)?;
        size.width = size.width.max(child_size.width);
        size.height = size.height.max(child_size.height);
    }
    Ok(size)
}

pub(super) fn measure_grid(
    children: &[Node<'_>],
    spec: GridSpec,
    row_spacing: u16,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    let mut height = 0u32;
    let mut row_start = 0;
    while row_start < children.len() {
        let row_end = (row_start + spec.columns()).min(children.len());
        let mut row_height = 0u16;
        for index in row_start..row_end {
            let width = spec.column_width(children, index % spec.columns(), cache, depth)?;
            let size = measure(
                &children[index],
                Constraints::loose(Size::new(width, UNBOUNDED)),
                cache,
                depth + 1,
            )?;
            row_height = row_height.max(size.height);
        }
        height = height.saturating_add(u32::from(row_height));
        if row_end < children.len() {
            height = height.saturating_add(u32::from(row_spacing));
        }
        row_start = row_end;
    }
    Ok(Size::new(
        spec.total_width(children, cache, depth)?,
        height.min(u32::from(u16::MAX)) as u16,
    ))
}

pub(super) fn measure_list(
    children: &[Node<'_>],
    row_height: u16,
    spacing: u16,
    max: Size,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    if children.len() > 1 && row_height == 0 && spacing == 0 {
        return Err(LayoutError::InvalidList);
    }
    let count = u32::try_from(children.len()).unwrap_or(u32::MAX);
    let height = count
        .saturating_mul(u32::from(row_height))
        .saturating_add(count.saturating_sub(1).saturating_mul(u32::from(spacing)));
    let width = if max.width != UNBOUNDED {
        max.width
    } else {
        measure_list_width(children, cache, depth)?
    };
    Ok(Size::new(width, height.min(u32::from(u16::MAX)) as u16))
}

fn measure_list_width(
    children: &[Node<'_>],
    cache: &mut TextCache,
    depth: u8,
) -> Result<u16, LayoutError> {
    let mut width = 0u16;
    for child in children {
        let size = measure(child, Constraints::unbounded(), cache, depth + 1)?;
        width = width.max(size.width);
    }
    Ok(width)
}

pub(super) fn measure_scroll(
    children: &[Node<'_>],
    axis: Axis,
    max: Size,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    if children.len() > 1 {
        return Err(LayoutError::InvalidScroll);
    }
    let Some(child) = children.first() else {
        return Ok(Size::ZERO);
    };
    let child_max = match axis {
        Axis::Vertical => Size::new(max.width, UNBOUNDED),
        Axis::Horizontal => Size::new(UNBOUNDED, max.height),
    };
    measure(child, Constraints::loose(child_max), cache, depth + 1)
}
