//! Insets, alignment, and axis selection.

use common::Region;

use super::constraint::{Size, UNBOUNDED};
use super::rect::LayoutRect;

/// A fixed-size four-sided inset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Insets {
    /// Space above the content.
    pub top: u16,
    /// Space to the right of the content.
    pub right: u16,
    /// Space below the content.
    pub bottom: u16,
    /// Space to the left of the content.
    pub left: u16,
}

impl Insets {
    /// No inset.
    pub const ZERO: Self = Self::new(0, 0, 0, 0);

    /// Creates insets in top, right, bottom, left order.
    pub const fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Equal insets on every edge.
    pub const fn all(px: u16) -> Self {
        Self::new(px, px, px, px)
    }

    /// Symmetric insets in horizontal, vertical order.
    pub const fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self::new(vertical, horizontal, vertical, horizontal)
    }

    /// Total horizontal inset.
    pub const fn horizontal(self) -> u16 {
        self.left.saturating_add(self.right)
    }

    /// Total vertical inset.
    pub const fn vertical(self) -> u16 {
        self.top.saturating_add(self.bottom)
    }

    /// Removes the insets from a size.
    pub const fn shrink(self, size: Size) -> Size {
        Size::new(
            shrink_available(size.width, self.horizontal()),
            shrink_available(size.height, self.vertical()),
        )
    }

    /// Applies the insets to a screen region.
    pub const fn inner(self, region: Region) -> Region {
        let left = if self.left < region.width {
            self.left
        } else {
            region.width
        };
        let right = if self.right < region.width - left {
            self.right
        } else {
            region.width - left
        };
        let top = if self.top < region.height {
            self.top
        } else {
            region.height
        };
        let bottom = if self.bottom < region.height - top {
            self.bottom
        } else {
            region.height - top
        };
        Region::new(
            region.x.saturating_add(left),
            region.y.saturating_add(top),
            region.width - left - right,
            region.height - top - bottom,
        )
    }

    /// Applies the insets to a layout rectangle.
    pub const fn inner_rect(self, rect: LayoutRect) -> LayoutRect {
        LayoutRect::new(
            rect.x.saturating_add(self.left as i32),
            rect.y.saturating_add(self.top as i32),
            shrink_available(rect.width, self.horizontal()),
            shrink_available(rect.height, self.vertical()),
        )
    }
}

const fn shrink_available(value: u16, inset: u16) -> u16 {
    if value == UNBOUNDED {
        UNBOUNDED
    } else {
        value.saturating_sub(inset)
    }
}

/// Cross-axis alignment inside a parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    /// Align to the start edge.
    Start,
    /// Center in the available space.
    Center,
    /// Align to the end edge.
    End,
}

impl Align {
    /// The offset of a child within available space.
    pub const fn offset(self, available: u16, size: u16) -> u16 {
        match self {
            Self::Start => 0,
            Self::Center => {
                if size < available {
                    (available - size) / 2
                } else {
                    0
                }
            }
            Self::End => available.saturating_sub(size),
        }
    }
}

/// The axis a scroll or list container moves along.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Left to right.
    Horizontal,
    /// Top to bottom.
    Vertical,
}
