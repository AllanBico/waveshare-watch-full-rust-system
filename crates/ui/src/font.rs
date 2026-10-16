//! Temporary 5x7 digit font for the monochrome clock.
//!
//! Only `0-9` and `:` are defined, and only at 1x. This exists to make the
//! Phase 3.3 pixel-budget test real and is not a general font.
// TODO(phase-5): replace with the theme font system.

/// Glyph width in pixels.
pub const GLYPH_WIDTH: u16 = 5;
/// Glyph height in pixels.
pub const GLYPH_HEIGHT: u16 = 7;

/// The seven rows of a glyph, top to bottom. Bit 4 is the leftmost pixel.
pub const fn glyph(character: u8) -> Option<[u8; GLYPH_HEIGHT as usize]> {
    let rows = match character {
        b'0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        b'1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        b'2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        b'3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        b'4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        b'5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        b'6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        b'7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        b'8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        b'9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        b':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        _ => return None,
    };
    Some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digits_and_colon_are_defined() {
        for &character in b"0123456789:" {
            assert!(glyph(character).is_some());
        }
        assert!(glyph(b'A').is_none());
    }

    #[test]
    fn glyph_rows_fit_five_bits() {
        for &character in b"0123456789:" {
            for row in glyph(character).unwrap() {
                assert!(row <= 0x1F, "row {row:#04x} exceeds 5 bits");
            }
        }
    }

    #[test]
    fn one_has_a_tall_stem() {
        let rows = glyph(b'1').unwrap();
        assert_eq!(rows[0], 0x04);
        assert_eq!(rows[6], 0x0E);
    }
}
