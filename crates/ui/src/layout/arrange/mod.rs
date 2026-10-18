//! The rectangle arrangement pass.

mod containers;
mod linear;

use common::Region;

use crate::text::TextCache;

use super::grid::GridSpec;
use super::insets::{Align, Axis};
use super::node::{Kind, Node};
use super::output::LayoutOutput;
use super::rect::LayoutRect;
use super::{check_depth, LayoutError};
use containers::{arrange_grid, arrange_list, arrange_scroll, arrange_stack};
use linear::{arrange_column, arrange_row};

/// Shared mutable state for one arrange traversal.
pub(crate) struct ArrangeContext<'c, 'e> {
    pub(super) cache: &'c mut TextCache,
    pub(super) out: &'c mut LayoutOutput<'e>,
}

impl<'c, 'e> ArrangeContext<'c, 'e> {
    /// Creates the context for one layout call.
    pub(crate) const fn new(cache: &'c mut TextCache, out: &'c mut LayoutOutput<'e>) -> Self {
        Self { cache, out }
    }
}

#[derive(Clone, Copy)]
struct BoxFrame {
    pub(super) content: LayoutRect,
    pub(super) clip: Region,
}

#[derive(Clone, Copy)]
struct LineSpec {
    pub(super) spacing: u16,
    pub(super) cross_align: Align,
}

#[derive(Clone, Copy)]
struct ListSpec {
    pub(super) row_height: u16,
    pub(super) spacing: u16,
    pub(super) offset: u16,
}

#[derive(Clone, Copy)]
struct ScrollSpec {
    pub(super) axis: Axis,
    pub(super) offset: u16,
}

#[derive(Clone, Copy)]
struct GridLayout {
    pub(super) spec: GridSpec,
    pub(super) row_spacing: u16,
}

impl LineSpec {
    const fn new(spacing: u16, cross_align: Align) -> Self {
        Self {
            spacing,
            cross_align,
        }
    }
}

impl ListSpec {
    const fn new(row_height: u16, spacing: u16, offset: u16) -> Self {
        Self {
            row_height,
            spacing,
            offset,
        }
    }
}

impl ScrollSpec {
    const fn new(axis: Axis, offset: u16) -> Self {
        Self { axis, offset }
    }
}

impl GridLayout {
    fn new(
        columns: u8,
        column_spacing: u16,
        row_spacing: u16,
        available: u16,
    ) -> Result<Self, LayoutError> {
        Ok(Self {
            spec: GridSpec::new(columns, column_spacing, available)?,
            row_spacing,
        })
    }
}

/// Arranges one node inside `rect`, clipped to `clip`.
pub(crate) fn arrange(
    node: &Node<'_>,
    rect: LayoutRect,
    clip: Region,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    check_depth(depth)?;
    context.out.push(node.id, rect, rect.visible(clip))?;
    let content = node.padding.inner_rect(rect);
    let frame = BoxFrame {
        content,
        clip: clip.intersection(content.visible(clip)),
    };
    arrange_kind(node, frame, context, depth)
}

fn arrange_kind(
    node: &Node<'_>,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    match node.kind {
        Kind::Leaf | Kind::Spacer => Ok(()),
        Kind::Row { .. } => arrange_row_node(node, frame, context, depth),
        Kind::Column { .. } => arrange_column_node(node, frame, context, depth),
        Kind::Stack { align } => arrange_stack(node.children, align, frame, context, depth),
        Kind::Grid { .. } => arrange_grid_node(node, frame, context, depth),
        Kind::List { .. } => arrange_list_node(node, frame, context, depth),
        Kind::Scroll { .. } => arrange_scroll_node(node, frame, context, depth),
    }
}

fn arrange_row_node(
    node: &Node<'_>,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let Kind::Row {
        spacing,
        cross_align,
    } = node.kind
    else {
        return Ok(());
    };
    arrange_row(
        node.children,
        LineSpec::new(spacing, cross_align),
        frame,
        context,
        depth,
    )
}

fn arrange_column_node(
    node: &Node<'_>,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let Kind::Column {
        spacing,
        cross_align,
    } = node.kind
    else {
        return Ok(());
    };
    arrange_column(
        node.children,
        LineSpec::new(spacing, cross_align),
        frame,
        context,
        depth,
    )
}

fn arrange_grid_node(
    node: &Node<'_>,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let Kind::Grid {
        columns,
        column_spacing,
        row_spacing,
    } = node.kind
    else {
        return Ok(());
    };
    let grid = GridLayout::new(columns, column_spacing, row_spacing, frame.content.width)?;
    arrange_grid(node.children, grid, frame, context, depth)
}

fn arrange_list_node(
    node: &Node<'_>,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let Kind::List {
        row_height,
        spacing,
        offset,
    } = node.kind
    else {
        return Ok(());
    };
    arrange_list(
        node.children,
        ListSpec::new(row_height, spacing, offset),
        frame,
        context,
        depth,
    )
}

fn arrange_scroll_node(
    node: &Node<'_>,
    frame: BoxFrame,
    context: &mut ArrangeContext<'_, '_>,
    depth: u8,
) -> Result<(), LayoutError> {
    let Kind::Scroll { axis, offset } = node.kind else {
        return Ok(());
    };
    arrange_scroll(
        node.children,
        ScrollSpec::new(axis, offset),
        frame,
        context,
        depth,
    )
}
