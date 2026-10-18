//! Signed layout rectangles and screen clipping.

use common::Region;

/// A rectangle in layout space.
///
/// Layout coordinates are signed so scroll content can extend past the top or
/// left edge before it is clipped to the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRect {
    /// Left edge in layout space.
    pub x: i32,
    /// Top edge in layout space.
    pub y: i32,
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
}

impl LayoutRect {
    /// Creates a layout rectangle.
    pub const fn new(x: i32, y: i32, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a layout rectangle from a screen region.
    pub const fn from_region(region: Region) -> Self {
        Self::new(
            region.x as i32,
            region.y as i32,
            region.width,
            region.height,
        )
    }

    /// The right edge, exclusive.
    pub const fn right(self) -> i32 {
        self.x.saturating_add(self.width as i32)
    }

    /// The bottom edge, exclusive.
    pub const fn bottom(self) -> i32 {
        self.y.saturating_add(self.height as i32)
    }

    /// The part of this rectangle visible inside `clip`, in screen space.
    pub fn visible(self, clip: Region) -> Region {
        let left = self.x.max(i32::from(clip.x));
        let top = self.y.max(i32::from(clip.y));
        let right = self.right().min(i32::from(clip.right()));
        let bottom = self.bottom().min(i32::from(clip.bottom()));
        if right <= left || bottom <= top {
            return Region::new(clip.x, clip.y, 0, 0);
        }
        Region::new(
            left as u16,
            top as u16,
            (right - left) as u16,
            (bottom - top) as u16,
        )
    }
}
