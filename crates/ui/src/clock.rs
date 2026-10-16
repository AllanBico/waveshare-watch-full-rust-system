//! Temporary monochrome clock renderer.
//!
//! Draws a fixed "HH:MM" into a [`MonoBuffer`] so the Phase 3.3 pixel-budget
//! test exercises real digit rendering. It hardcodes a 5x7 font and no layout.
// TODO(phase-5): replace with the clock app on the shared render/font system.

use common::Region;

use crate::font::{glyph, GLYPH_HEIGHT, GLYPH_WIDTH};
use crate::mono::MonoBuffer;

/// Gap between glyphs, in unscaled pixels.
const GLYPH_GAP: u16 = 1;
/// Character cells in "HH:MM".
const CELL_COUNT: u16 = 5;

/// The rendered size of "HH:MM" at `scale`.
pub fn clock_size(scale: u16) -> (u16, u16) {
    let width = (CELL_COUNT * GLYPH_WIDTH + (CELL_COUNT - 1) * GLYPH_GAP) * scale;
    (width, GLYPH_HEIGHT * scale)
}

/// The region "HH:MM" occupies at `origin` and `scale`.
pub fn clock_region(origin: (u16, u16), scale: u16) -> Region {
    let (width, height) = clock_size(scale);
    Region::new(origin.0, origin.1, width, height)
}

/// Draws "HH:MM" into `buffer` with its top-left at `origin`.
pub fn draw_time(buffer: &mut MonoBuffer, hour: u8, minute: u8, origin: (u16, u16), scale: u16) {
    let text = [
        b'0' + hour / 10,
        b'0' + hour % 10,
        b':',
        b'0' + minute / 10,
        b'0' + minute % 10,
    ];
    for (cell, &character) in text.iter().enumerate() {
        let x = origin.0 + cell as u16 * (GLYPH_WIDTH + GLYPH_GAP) * scale;
        draw_glyph(buffer, character, (x, origin.1), scale);
    }
}

/// Draws one glyph as `scale` by `scale` blocks.
fn draw_glyph(buffer: &mut MonoBuffer, character: u8, origin: (u16, u16), scale: u16) {
    let Some(rows) = glyph(character) else {
        return;
    };
    for (row_index, row) in rows.iter().enumerate() {
        for col in 0..GLYPH_WIDTH {
            if row & (0b1_0000 >> col) == 0 {
                continue;
            }
            let x = origin.0 + col * scale;
            let y = origin.1 + row_index as u16 * scale;
            buffer.fill_rect(Region::new(x, y, scale, scale), true);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PANEL_WIDTH: u16 = 410;
    const PANEL_HEIGHT: u16 = 502;
    const SCALE: u16 = 3;

    fn centered_origin() -> (u16, u16) {
        let (width, height) = clock_size(SCALE);
        ((PANEL_WIDTH - width) / 2, (PANEL_HEIGHT - height) / 2)
    }

    #[test]
    fn clock_size_scales_with_the_scale() {
        assert_eq!(clock_size(1), (29, 7));
        assert_eq!(clock_size(3), (87, 21));
    }

    #[test]
    fn drawn_clock_stays_under_the_pixel_budget() {
        let mut buffer = MonoBuffer::new(PANEL_WIDTH, PANEL_HEIGHT);
        draw_time(&mut buffer, 10, 9, centered_origin(), SCALE);

        let lit = buffer.lit_pixels();
        let total = buffer.total_pixels();
        assert!(lit > 0, "clock drew nothing");
        assert!(lit * 100 < total * 5, "{lit} of {total} lit exceeds 5%");
    }

    #[test]
    fn nothing_is_drawn_outside_the_clock_region() {
        let mut buffer = MonoBuffer::new(PANEL_WIDTH, PANEL_HEIGHT);
        let origin = centered_origin();
        draw_time(&mut buffer, 10, 9, origin, SCALE);

        let region = clock_region(origin, SCALE);
        let mut outside_lit = 0;
        for y in 0..PANEL_HEIGHT {
            for x in 0..PANEL_WIDTH {
                let inside = x >= region.x
                    && x < region.x + region.width
                    && y >= region.y
                    && y < region.y + region.height;
                if !inside && buffer.is_lit(x, y) {
                    outside_lit += 1;
                }
            }
        }
        assert_eq!(outside_lit, 0);
    }
}
