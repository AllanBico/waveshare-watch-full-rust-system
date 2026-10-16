//! Color types.

/// A 16-bit RGB565 color.
///
/// Serialized big-endian (high byte first) when written to a panel, matching
/// the CO5300's wire order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Rgb565(u16);

impl Rgb565 {
    /// Black.
    pub const BLACK: Self = Self(0x0000);
    /// White.
    pub const WHITE: Self = Self(0xFFFF);
    /// Pure red.
    pub const RED: Self = Self(0xF800);
    /// Pure green.
    pub const GREEN: Self = Self(0x07E0);
    /// Pure blue.
    pub const BLUE: Self = Self(0x001F);

    /// Wraps a raw RGB565 value.
    pub const fn from_raw(raw: u16) -> Self {
        Self(raw)
    }

    /// The raw RGB565 value.
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// The value as the two wire bytes, high byte first.
    pub const fn to_be_bytes(self) -> [u8; 2] {
        self.0.to_be_bytes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn named_colors_have_expected_bits() {
        assert_eq!(Rgb565::RED.raw(), 0xF800);
        assert_eq!(Rgb565::GREEN.raw(), 0x07E0);
        assert_eq!(Rgb565::BLUE.raw(), 0x001F);
        assert_eq!(Rgb565::BLACK.raw(), 0x0000);
        assert_eq!(Rgb565::WHITE.raw(), 0xFFFF);
    }

    #[test]
    fn wire_order_is_high_byte_first() {
        assert_eq!(Rgb565::RED.to_be_bytes(), [0xF8, 0x00]);
        assert_eq!(Rgb565::BLUE.to_be_bytes(), [0x00, 0x1F]);
    }
}
