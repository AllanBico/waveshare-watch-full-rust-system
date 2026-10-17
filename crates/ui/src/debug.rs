//! Temporary debug text for bring-up viewers.
//!
//! Reuses the 5x7 glyph set. This is not the Phase 5 font system.

use crate::clock::draw_glyph;
use crate::font::{GLYPH_HEIGHT, GLYPH_WIDTH};
use crate::mono::MonoBuffer;

const GLYPH_GAP: u16 = 1;

/// The rendered size of `text` at `scale`.
pub fn text_size(text: &str, scale: u16) -> (u16, u16) {
    let cells = text.len() as u16;
    let width = (cells * GLYPH_WIDTH + cells.saturating_sub(1) * GLYPH_GAP) * scale;
    (width, GLYPH_HEIGHT * scale)
}

/// Draws `text` into `buffer` with its top-left at `origin`.
pub fn draw_text(buffer: &mut MonoBuffer, text: &str, origin: (u16, u16), scale: u16) {
    for (cell, character) in text.bytes().enumerate() {
        let x = origin.0 + cell as u16 * (GLYPH_WIDTH + GLYPH_GAP) * scale;
        draw_glyph(buffer, character.to_ascii_uppercase(), (x, origin.1), scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_uses_cells_and_gaps() {
        assert_eq!(text_size("KV", 1), (11, 7));
        assert_eq!(text_size("KV", 2), (22, 14));
        assert_eq!(text_size("", 2), (0, 14));
    }

    #[test]
    fn text_lights_pixels() {
        let mut buffer = MonoBuffer::new(64, 16);
        draw_text(&mut buffer, "KV OK", (0, 0), 1);
        assert!(buffer.lit_pixels() > 0);
    }

    #[test]
    fn lowercase_is_drawn_as_uppercase() {
        let mut lower = MonoBuffer::new(64, 16);
        let mut upper = MonoBuffer::new(64, 16);
        draw_text(&mut lower, "kv", (0, 0), 1);
        draw_text(&mut upper, "KV", (0, 0), 1);
        assert_eq!(lower.lit_pixels(), upper.lit_pixels());
    }
}
