//! Layout tree nodes and container kinds.

use crate::text::TextAlign;
use crate::theme::TextStyle;

use super::constraint::Length;
use super::insets::{Align, Axis, Insets};

/// A caller-assigned stable node identifier.
pub type NodeId = u16;

/// How a container arranges its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A plain leaf with no children.
    Leaf,
    /// Flexible empty space.
    Spacer,
    /// Children left to right.
    Row {
        /// Gap between adjacent children.
        spacing: u16,
        /// Cross-axis alignment.
        cross_align: Align,
    },
    /// Children top to bottom.
    Column {
        /// Gap between adjacent children.
        spacing: u16,
        /// Cross-axis alignment.
        cross_align: Align,
    },
    /// Children stacked on top of each other.
    Stack {
        /// Alignment on both axes.
        align: Align,
    },
    /// A uniform grid of cells.
    Grid {
        /// Number of columns; must be non-zero.
        columns: u8,
        /// Gap between columns.
        column_spacing: u16,
        /// Gap between rows.
        row_spacing: u16,
    },
    /// A virtualized list of fixed-height rows.
    List {
        /// Height of every row.
        row_height: u16,
        /// Gap between rows.
        spacing: u16,
        /// Scroll offset from the top.
        offset: u16,
    },
    /// A clipped viewport over one oversized child.
    Scroll {
        /// Scroll direction.
        axis: Axis,
        /// Scroll offset along the axis.
        offset: u16,
    },
}

/// Leaf content drawn inside a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Content<'a> {
    /// No content.
    None,
    /// Wrapped text drawn from the node origin.
    Text {
        /// Text to draw.
        text: &'a str,
        /// Compiled-in text style.
        style: TextStyle,
        /// Horizontal alignment inside the node.
        align: TextAlign,
    },
}

/// One node in an immutable layout tree.
#[derive(Debug, Clone, Copy)]
pub struct Node<'a> {
    /// Stable caller-assigned identifier.
    pub id: NodeId,
    /// Width rule.
    pub width: Length,
    /// Height rule.
    pub height: Length,
    /// Inner padding.
    pub padding: Insets,
    /// Container behavior.
    pub kind: Kind,
    /// Leaf content.
    pub content: Content<'a>,
    /// Child nodes in paint order.
    pub children: &'a [Node<'a>],
}

impl<'a> Node<'a> {
    /// Creates a container node with no content.
    pub const fn container(
        id: NodeId,
        width: Length,
        height: Length,
        kind: Kind,
        children: &'a [Node<'a>],
    ) -> Self {
        Self {
            id,
            width,
            height,
            padding: Insets::ZERO,
            kind,
            content: Content::None,
            children,
        }
    }

    /// Creates a start-aligned text leaf.
    pub const fn text(
        id: NodeId,
        width: Length,
        height: Length,
        text: &'a str,
        style: TextStyle,
    ) -> Self {
        Self {
            id,
            width,
            height,
            padding: Insets::ZERO,
            kind: Kind::Leaf,
            content: Content::Text {
                text,
                style,
                align: TextAlign::Start,
            },
            children: &[],
        }
    }

    /// Sets the horizontal alignment of text content.
    pub const fn with_text_align(mut self, align: TextAlign) -> Self {
        match self.content {
            Content::Text { text, style, .. } => {
                self.content = Content::Text { text, style, align };
            }
            Content::None => {}
        }
        self
    }

    /// Creates an empty flexible spacer.
    pub const fn spacer(id: NodeId, width: Length, height: Length) -> Self {
        Self {
            id,
            width,
            height,
            padding: Insets::ZERO,
            kind: Kind::Spacer,
            content: Content::None,
            children: &[],
        }
    }

    /// Sets the node padding.
    pub const fn with_padding(mut self, padding: Insets) -> Self {
        self.padding = padding;
        self
    }
}
