//! Integer layout for the monochrome UI engine.
//!
//! Layout is pure and deterministic: a measure pass computes intrinsic sizes,
//! then an arrange pass assigns rectangles. Painting never triggers layout.

mod arrange;
mod constraint;
mod fill;
mod grid;
mod insets;
mod measure;
mod node;
mod output;
mod rect;
#[cfg(test)]
mod tests;

pub use constraint::{Constraints, Length, Size, UNBOUNDED};
pub use insets::{Align, Axis, Insets};
pub use node::{Content, Kind, Node, NodeId};
pub use output::LayoutEntry;
pub use rect::LayoutRect;

use common::Region;
use core::fmt;

use crate::text::TextCache;

/// Maximum container nesting depth, counting the root as one.
pub const MAX_DEPTH: u8 = 32;

/// A layout failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutError {
    /// The tree exceeded [`MAX_DEPTH`].
    DepthExceeded,
    /// The caller supplied too little entry storage.
    OutputFull,
    /// A grid was declared with zero columns.
    InvalidGrid,
    /// A scroll node had more than one child.
    InvalidScroll,
    /// A fixed-height list could not advance between rows.
    InvalidList,
}

impl fmt::Display for LayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::DepthExceeded => "layout depth exceeded",
            Self::OutputFull => "layout output full",
            Self::InvalidGrid => "grid requires at least one column",
            Self::InvalidScroll => "scroll requires exactly one child",
            Self::InvalidList => "list requires a non-zero row stride",
        })
    }
}

/// Measures and arranges `root` inside `viewport` after applying `safe_area`.
///
/// Entries are written in pre-order. Off-screen list rows are skipped. The
/// returned count is the number of entries written.
pub fn layout(
    root: &Node<'_>,
    viewport: Region,
    safe_area: Insets,
    cache: &mut TextCache,
    entries: &mut [LayoutEntry],
) -> Result<usize, LayoutError> {
    let safe = safe_area.inner(viewport);
    let constraints = Constraints::tight(Size::new(safe.width, safe.height));
    measure::measure(root, constraints, cache, 1)?;
    let mut output = output::LayoutOutput::new(entries);
    let mut context = arrange::ArrangeContext::new(cache, &mut output);
    arrange::arrange(root, LayoutRect::from_region(safe), safe, &mut context, 1)?;
    Ok(output.len())
}

pub(crate) fn check_depth(depth: u8) -> Result<(), LayoutError> {
    if depth > MAX_DEPTH {
        Err(LayoutError::DepthExceeded)
    } else {
        Ok(())
    }
}
