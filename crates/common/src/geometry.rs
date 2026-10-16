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
}
