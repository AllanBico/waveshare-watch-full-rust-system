//! Rectangular pixel geometry.

/// An axis-aligned rectangle in physical pixels.
///
/// Coordinates are top-left based. The panel applies its own column/row
/// offsets; a `Region` is always in logical screen space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Region {
    /// Left edge.
    pub x: u16,
    /// Top edge.
    pub y: u16,
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
}

impl Region {
    /// Creates a region.
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// The full `width` by `height` screen.
    pub const fn full(width: u16, height: u16) -> Self {
        Self::new(0, 0, width, height)
    }

    /// Whether the region covers no pixels.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }

    /// The number of pixels the region covers.
    pub const fn pixel_count(self) -> u32 {
        self.width as u32 * self.height as u32
    }

    /// The size of an RGB565 bitmap for this region, in bytes.
    pub const fn rgb565_len(self) -> usize {
        self.pixel_count() as usize * 2
    }

    /// Whether this region lies entirely within a `width` by `height` screen.
    pub const fn is_within(self, width: u16, height: u16) -> bool {
        self.x as u32 + self.width as u32 <= width as u32
            && self.y as u32 + self.height as u32 <= height as u32
    }

    /// The right edge, exclusive.
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.width)
    }

    /// The bottom edge, exclusive.
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.height)
    }

    /// Whether a point lies inside the region.
    pub const fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// The overlap of two regions, or an empty region when they do not overlap.
    pub const fn intersection(self, other: Self) -> Self {
        let x = if self.x > other.x { self.x } else { other.x };
        let y = if self.y > other.y { self.y } else { other.y };
        let right = if self.right() < other.right() {
            self.right()
        } else {
            other.right()
        };
        let bottom = if self.bottom() < other.bottom() {
            self.bottom()
        } else {
            other.bottom()
        };
        if right <= x || bottom <= y {
            Self::new(x, y, 0, 0)
        } else {
            Self::new(x, y, right - x, bottom - y)
        }
    }

    /// The smallest region covering both inputs.
    pub const fn union(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }
        let x = if self.x < other.x { self.x } else { other.x };
        let y = if self.y < other.y { self.y } else { other.y };
        let right = if self.right() > other.right() {
            self.right()
        } else {
            other.right()
        };
        let bottom = if self.bottom() > other.bottom() {
            self.bottom()
        } else {
            other.bottom()
        };
        Self::new(x, y, right - x, bottom - y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_covers_the_screen() {
        let region = Region::full(410, 502);
        assert_eq!(region.pixel_count(), 410 * 502);
        assert!(region.is_within(410, 502));
    }

    #[test]
    fn empty_regions_report_zero_pixels() {
        assert!(Region::new(0, 0, 0, 10).is_empty());
        assert!(Region::new(0, 0, 10, 0).is_empty());
        assert_eq!(Region::new(0, 0, 0, 10).pixel_count(), 0);
    }

    #[test]
    fn rgb565_len_is_two_bytes_per_pixel() {
        assert_eq!(Region::new(0, 0, 16, 16).rgb565_len(), 512);
    }

    #[test]
    fn containment_checks_both_edges() {
        assert!(Region::new(400, 0, 10, 10).is_within(410, 10));
        assert!(!Region::new(401, 0, 10, 10).is_within(410, 10));
        assert!(!Region::new(0, 5, 10, 6).is_within(10, 10));
    }

    #[test]
    fn edges_are_exclusive() {
        let region = Region::new(2, 3, 4, 5);
        assert_eq!(region.right(), 6);
        assert_eq!(region.bottom(), 8);
        assert!(region.contains(2, 3));
        assert!(region.contains(5, 7));
        assert!(!region.contains(6, 7));
        assert!(!region.contains(5, 8));
    }

    #[test]
    fn intersection_is_the_overlap() {
        let a = Region::new(0, 0, 10, 10);
        let b = Region::new(5, 5, 10, 10);
        assert_eq!(a.intersection(b), Region::new(5, 5, 5, 5));
        assert!(a.intersection(Region::new(20, 20, 5, 5)).is_empty());
    }

    #[test]
    fn union_covers_both_inputs() {
        let a = Region::new(2, 3, 4, 5);
        let b = Region::new(8, 1, 2, 2);
        assert_eq!(a.union(b), Region::new(2, 1, 8, 7));
        assert_eq!(a.union(Region::new(0, 0, 0, 0)), a);
    }
}
