//! Stack, grid, list, and scroll arrangement.

use super::super::constraint::{Constraints, Size, UNBOUNDED};
use super::super::insets::{Align, Axis};
use super::super::measure::measure;
use super::super::node::Node;
use super::super::rect::LayoutRect;
use super::super::LayoutError;
use super::{arrange, ArrangeContext, BoxFrame, GridLayout, ListSpec, ScrollSpec};

pub(super) fn arrange_stack(
    children: &[Node<'_>],
    align: Align,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let max = Size::new(frame.content.width, frame.content.height);
    for child in children {
        let size = measure(child, Constraints::loose(max), context.cache, depth + 1)?;
        let x = frame
            .content
            .x
            .saturating_add(i32::from(align.offset(frame.content.width, size.width)));
        let y = frame
            .content
            .y
            .saturating_add(i32::from(align.offset(frame.content.height, size.height)));
        let rect = LayoutRect::new(x, y, size.width, size.height);
        arrange(child, rect, frame.clip, context, depth + 1)?;
    }
    Ok(())
}

pub(super) fn arrange_grid(
    children: &[Node<'_>],
    grid: GridLayout,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let columns = grid.spec.columns();
    let mut y = frame.content.y;
    let mut start = 0;
    while start < children.len() {
        let end = (start + columns).min(children.len());
        let row = GridRow {
            grid,
            frame,
            start,
            end,
            y,
        };
        let height = row.place(children, context, depth)?;
        y = y
            .saturating_add(i32::from(height))
            .saturating_add(i32::from(grid.row_spacing));
        start = end;
    }
    Ok(())
}

struct GridRow {
    grid: GridLayout,
    frame: BoxFrame,
    start: usize,
    end: usize,
    y: i32,
}

impl GridRow {
    fn place(
        self,
        children: &[Node<'_>],
        context: &mut ArrangeContext<'_, '_>,
        depth: u8,
    ) -> Result<u16, LayoutError> {
        let height = self.grid.row_height(children, self.start, context, depth)?;
        let mut x = self.frame.content.x;
        for index in self.start..self.end {
            let track = self.grid.spec.column_width(
                children,
                index % self.grid.spec.columns(),
                context.cache,
                depth,
            )?;
            let child = &children[index];
            let size = measure(
                child,
                Constraints::loose(Size::new(track, UNBOUNDED)),
                context.cache,
                depth + 1,
            )?;
            let width = if child.width.is_fill() {
                track
            } else {
                size.width.min(track)
            };
            let child_height = if child.height.is_fill() {
                height
            } else {
                size.height
            };
            let rect = LayoutRect::new(x, self.y, width, child_height);
            arrange(child, rect, self.frame.clip, context, depth + 1)?;
            x = x
                .saturating_add(i32::from(track))
                .saturating_add(i32::from(self.grid.spec.column_spacing()));
        }
        Ok(height)
    }
}

impl GridLayout {
    fn row_height(
        self,
        children: &[Node<'_>],
        start: usize,
        context: &mut ArrangeContext<'_, '_>,
        depth: u8,
    ) -> Result<u16, LayoutError> {
        let end = (start + self.spec.columns()).min(children.len());
        let mut height = 0u16;
        for index in start..end {
            let track = self.spec.column_width(
                children,
                index % self.spec.columns(),
                context.cache,
                depth,
            )?;
            let size = measure(
                &children[index],
                Constraints::loose(Size::new(track, UNBOUNDED)),
                context.cache,
                depth + 1,
            )?;
            height = height.max(size.height);
        }
        Ok(height)
    }
}

pub(super) fn arrange_list(
    children: &[Node<'_>],
    list: ListSpec,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let stride = u32::from(list.row_height).saturating_add(u32::from(list.spacing));
    if children.len() > 1 && stride == 0 {
        return Err(LayoutError::InvalidList);
    }
    let count = u32::try_from(children.len()).unwrap_or(u32::MAX);
    let total = count
        .saturating_mul(u32::from(list.row_height))
        .saturating_add(
            count
                .saturating_sub(1)
                .saturating_mul(u32::from(list.spacing)),
        );
    let viewport = u32::from(frame.content.height);
    let offset = u32::from(list.offset).min(total.saturating_sub(viewport));
    let (start, end) = visible_rows(offset, viewport, stride, count);
    for index in start..end {
        if !row_is_visible(index, list.row_height, frame, offset, stride) {
            continue;
        }
        let row_y = listed_row_y(index, frame, offset, stride);
        let rect = LayoutRect::new(frame.content.x, row_y, frame.content.width, list.row_height);
        arrange(
            &children[index as usize],
            rect,
            frame.clip,
            context,
            depth + 1,
        )?;
    }
    Ok(())
}

fn visible_rows(offset: u32, viewport: u32, stride: u32, count: u32) -> (u32, u32) {
    if stride == 0 || viewport == 0 {
        return (0, 0);
    }
    let start = offset / stride;
    let end = offset.saturating_add(viewport).saturating_add(stride - 1) / stride;
    (start.min(count), end.min(count))
}

fn row_is_visible(index: u32, row_height: u16, frame: BoxFrame, offset: u32, stride: u32) -> bool {
    let top = i64::from(frame.content.y);
    let bottom = top + i64::from(frame.content.height);
    let row = top + i64::from(index) * i64::from(stride) - i64::from(offset);
    row < bottom && row + i64::from(row_height) > top
}

fn listed_row_y(index: u32, frame: BoxFrame, offset: u32, stride: u32) -> i32 {
    let row = i64::from(frame.content.y) + i64::from(index) * i64::from(stride) - i64::from(offset);
    row.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

pub(super) fn arrange_scroll(
    children: &[Node<'_>],
    scroll: ScrollSpec,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    if children.len() > 1 {
        return Err(LayoutError::InvalidScroll);
    }
    let Some(child) = children.first() else {
        return Ok(());
    };
    let child_max = match scroll.axis {
        Axis::Vertical => Size::new(frame.content.width, UNBOUNDED),
        Axis::Horizontal => Size::new(UNBOUNDED, frame.content.height),
    };
    let size = measure(
        child,
        Constraints::loose(child_max),
        context.cache,
        depth + 1,
    )?;
    let offset = scroll_offset(scroll.axis, scroll.offset, size, frame);
    let rect = scroll_rect(scroll.axis, frame, size, offset);
    arrange(child, rect, frame.clip, context, depth + 1)
}

fn scroll_rect(axis: Axis, frame: BoxFrame, size: Size, offset: u16) -> LayoutRect {
    match axis {
        Axis::Vertical => LayoutRect::new(
            frame.content.x,
            frame.content.y.saturating_sub(i32::from(offset)),
            frame.content.width,
            size.height,
        ),
        Axis::Horizontal => LayoutRect::new(
            frame.content.x.saturating_sub(i32::from(offset)),
            frame.content.y,
            size.width,
            frame.content.height,
        ),
    }
}

fn scroll_offset(axis: Axis, offset: u16, size: Size, frame: BoxFrame) -> u16 {
    let main = match axis {
        Axis::Vertical => size.height,
        Axis::Horizontal => size.width,
    };
    let viewport = match axis {
        Axis::Vertical => frame.content.height,
        Axis::Horizontal => frame.content.width,
    };
    offset.min(main.saturating_sub(viewport))
}
