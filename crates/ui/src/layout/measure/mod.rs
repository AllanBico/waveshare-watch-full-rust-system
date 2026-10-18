//! The intrinsic measurement pass.

mod containers;
mod linear;

use crate::text::{measure_text, TextCache};
use crate::theme::TextStyle;

use super::constraint::{resolve, Constraints, Length, Size, UNBOUNDED};
use super::grid::GridSpec;
use super::node::{Content, Kind, Node};
use super::{check_depth, LayoutError};
use containers::{measure_grid, measure_list, measure_scroll, measure_stack};
use linear::{measure_column, measure_row};

/// Measures one node under `constraints`.
pub(crate) fn measure(
    node: &Node<'_>,
    constraints: Constraints,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    check_depth(depth)?;
    let width = explicit_axis(node.width, constraints.max.width, constraints.min.width);
    let height = explicit_axis(node.height, constraints.max.height, constraints.min.height);
    let content_max = node.padding.shrink(Size::new(
        width.unwrap_or(constraints.max.width),
        height.unwrap_or(constraints.max.height),
    ));
    let content = measure_content(node, content_max, cache, depth)?;
    let fit = Size::new(
        content.width.saturating_add(node.padding.horizontal()),
        content.height.saturating_add(node.padding.vertical()),
    );
    let size = Size::new(
        resolve(node.width, constraints.max.width, fit.width),
        resolve(node.height, constraints.max.height, fit.height),
    );
    Ok(constraints.constrain(size))
}

fn explicit_axis(length: Length, max: u16, min: u16) -> Option<u16> {
    match length {
        Length::Fit => None,
        Length::Fill(_) | Length::Percent(_) if max == UNBOUNDED => None,
        _ => {
            let resolved = resolve(length, max, 0);
            let low = min.min(max);
            Some(resolved.max(low).min(max))
        }
    }
}

fn measure_content(
    node: &Node<'_>,
    max: Size,
    cache: &mut TextCache,
    depth: u8,
) -> Result<Size, LayoutError> {
    if let Content::Text { text, style, .. } = node.content {
        return Ok(measure_text_content(text, style, max, cache));
    }
    match node.kind {
        Kind::Leaf | Kind::Spacer => Ok(Size::ZERO),
        Kind::Row { spacing, .. } => measure_row(node.children, spacing, max, cache, depth),
        Kind::Column { spacing, .. } => measure_column(node.children, spacing, max, cache, depth),
        Kind::Stack { .. } => measure_stack(node.children, max, cache, depth),
        Kind::Grid {
            columns,
            column_spacing,
            row_spacing,
        } => {
            let spec = GridSpec::new(columns, column_spacing, max.width)?;
            measure_grid(node.children, spec, row_spacing, cache, depth)
        }
        Kind::List {
            row_height,
            spacing,
            ..
        } => measure_list(node.children, row_height, spacing, max, cache, depth),
        Kind::Scroll { axis, .. } => measure_scroll(node.children, axis, max, cache, depth),
    }
}

fn measure_text_content(text: &str, style: TextStyle, max: Size, cache: &mut TextCache) -> Size {
    let wrap = if max.width == UNBOUNDED { 0 } else { max.width };
    let metrics = measure_text(text, &style, wrap, cache);
    Size::new(metrics.width.min(max.width), metrics.height)
}
