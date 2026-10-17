//! Luminance values shared by both monochrome pixel formats.

/// An 8-bit luminance value where `0` is black and `255` is white.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Luma(u8);

impl Luma {
    /// Black.
    pub const BLACK: Self = Self(0);
    /// White.
    pub const WHITE: Self = Self(255);
    /// The midpoint used when quantizing to one bit.
    pub const MIDPOINT: u8 = 128;

    /// Creates a luminance value.
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// The raw 8-bit value.
    pub const fn raw(self) -> u8 {
        self.0
    }

    /// Whether this value quantizes to white in 1-bit mode.
    pub const fn is_lit(self) -> bool {
        self.0 >= Self::MIDPOINT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_and_white_are_the_extremes() {
        assert_eq!(Luma::BLACK.raw(), 0);
        assert_eq!(Luma::WHITE.raw(), 255);
        assert!(!Luma::BLACK.is_lit());
        assert!(Luma::WHITE.is_lit());
    }

    #[test]
    fn the_midpoint_is_lit() {
        assert!(!Luma::new(127).is_lit());
        assert!(Luma::new(128).is_lit());
    }
}
