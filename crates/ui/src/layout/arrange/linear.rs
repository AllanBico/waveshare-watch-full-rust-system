//! Row and column arrangement.

use super::super::constraint::{Constraints, Size, UNBOUNDED};
use super::super::fill::{gap_total, FillCursor};
use super::super::measure::measure;
use super::super::node::Node;
use super::super::rect::LayoutRect;
use super::super::LayoutError;
use super::{arrange, ArrangeContext, BoxFrame, LineSpec};

pub(super) fn arrange_row(
    children: &[Node<'_>],
    spec: LineSpec,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let (remaining, fill_weight) =
        row_fill_space(children, spec.spacing, frame.content, context, depth)?;
    let mut cursor = FillCursor::new(remaining, fill_weight);
    let mut x = frame.content.x;
    for child in children {
        let is_fill = frame.content.width != UNBOUNDED && child.width.is_fill();
        let width = if is_fill {
            cursor.take(child.width.weight())
        } else {
            frame.content.width
        };
        let max = Constraints::loose(Size::new(width, frame.content.height));
        let size = measure(child, max, context.cache, depth + 1)?;
        let width = if is_fill { width } else { size.width };
        let offset = spec.cross_align.offset(frame.content.height, size.height);
        let rect = LayoutRect::new(
            x,
            frame.content.y.saturating_add(i32::from(offset)),
            width,
            size.height,
        );
        arrange(child, rect, frame.clip, context, depth + 1)?;
        x = x
            .saturating_add(i32::from(width))
            .saturating_add(i32::from(spec.spacing));
    }
    Ok(())
}

fn row_fill_space(
    children: &[Node<'_>],
    spacing: u16,
    content: LayoutRect,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(u16, u32), LayoutError> {
    let available = content.width;
    let mut fixed = 0u32;
    let mut weight = 0u32;
    for child in children {
        if available != UNBOUNDED && child.width.is_fill() {
            weight = weight.saturating_add(u32::from(child.width.weight()));
            continue;
        }
        let max = Constraints::loose(Size::new(available, content.height));
        let size = measure(child, max, context.cache, depth + 1)?;
        fixed = fixed.saturating_add(u32::from(size.width));
    }
    fixed = fixed.saturating_add(gap_total(children.len(), spacing));
    let used = fixed.min(u32::from(available)) as u16;
    Ok((available.saturating_sub(used), weight))
}

pub(super) fn arrange_column(
    children: &[Node<'_>],
    spec: LineSpec,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let (remaining, fill_weight) =
        column_fill_space(children, spec.spacing, frame.content, context, depth)?;
    let mut cursor = FillCursor::new(remaining, fill_weight);
    let mut y = frame.content.y;
    for child in children {
        let is_fill = frame.content.height != UNBOUNDED && child.height.is_fill();
        let height = if is_fill {
            cursor.take(child.height.weight())
        } else {
            frame.content.height
        };
        let max = Constraints::loose(Size::new(frame.content.width, height));
        let size = measure(child, max, context.cache, depth + 1)?;
        let height = if is_fill { height } else { size.height };
        let offset = spec.cross_align.offset(frame.content.width, size.width);
        let rect = LayoutRect::new(
            frame.content.x.saturating_add(i32::from(offset)),
            y,
            size.width,
            height,
        );
        arrange(child, rect, frame.clip, context, depth + 1)?;
        y = y
            .saturating_add(i32::from(height))
            .saturating_add(i32::from(spec.spacing));
    }
    Ok(())
}

fn column_fill_space(
    children: &[Node<'_>],
    spacing: u16,
    content: LayoutRect,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(u16, u32), LayoutError> {
    let available = content.height;
    let mut fixed = 0u32;
    let mut weight = 0u32;
    for child in children {
        if available != UNBOUNDED && child.height.is_fill() {
            weight = weight.saturating_add(u32::from(child.height.weight()));
            continue;
        }
        let max = Constraints::loose(Size::new(content.width, available));
        let size = measure(child, max, context.cache, depth + 1)?;
        fixed = fixed.saturating_add(u32::from(size.height));
    }
    fixed = fixed.saturating_add(gap_total(children.len(), spacing));
    let used = fixed.min(u32::from(available)) as u16;
    Ok((available.saturating_sub(used), weight))
}
