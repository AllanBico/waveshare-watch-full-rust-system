//! Temporary monochrome clock and number renderer.
//!
//! Draws "HH:MM" and a zero-padded number into a [`MonoBuffer`] so the display
//! bring-up can show real stats. It hardcodes a 5x7 font and no layout.
// TODO(phase-5): replace with the clock app on the shared render/font system.

use common::Region;

use crate::font::{glyph, GLYPH_HEIGHT, GLYPH_WIDTH};
use crate::mono::MonoBuffer;

/// Gap between glyphs, in unscaled pixels.
const GLYPH_GAP: u16 = 1;
/// Character cells in "HH:MM:SS".
const CELL_COUNT: u16 = 8;

/// The rendered size of `cells` glyphs at `scale`.
fn text_size(cells: u16, scale: u16) -> (u16, u16) {
    let width = (cells * GLYPH_WIDTH + cells.saturating_sub(1) * GLYPH_GAP) * scale;
    (width, GLYPH_HEIGHT * scale)
}

/// The rendered size of "HH:MM:SS" at `scale`.
pub fn clock_size(scale: u16) -> (u16, u16) {
    text_size(CELL_COUNT, scale)
}

/// The rendered size of `digits` decimal digits at `scale`.
pub fn digits_size(digits: u8, scale: u16) -> (u16, u16) {
    text_size(u16::from(digits), scale)
}

/// The region "HH:MM:SS" occupies at `origin` and `scale`.
pub fn clock_region(origin: (u16, u16), scale: u16) -> Region {
    let (width, height) = clock_size(scale);
    Region::new(origin.0, origin.1, width, height)
}

/// Draws "HH:MM:SS" into `buffer` with its top-left at `origin`.
pub fn draw_time(
    buffer: &mut MonoBuffer,
    hour: u8,
    minute: u8,
    second: u8,
    origin: (u16, u16),
    scale: u16,
) {
    let text = [
        b'0' + hour / 10,
        b'0' + hour % 10,
        b':',
        b'0' + minute / 10,
        b'0' + minute % 10,
        b':',
        b'0' + second / 10,
        b'0' + second % 10,
    ];
    for (cell, &character) in text.iter().enumerate() {
        let x = origin.0 + cell as u16 * (GLYPH_WIDTH + GLYPH_GAP) * scale;
        draw_glyph(buffer, character, (x, origin.1), scale);
    }
}

/// Draws `value` as `digits` zero-padded decimal digits at `origin`.
pub fn draw_digits(
    buffer: &mut MonoBuffer,
    value: u16,
    digits: u8,
    origin: (u16, u16),
    scale: u16,
) {
    let mut remaining = u32::from(value);
    let mut divisor = 10u32.pow(u32::from(digits.saturating_sub(1)));
    let mut x = origin.0;
    for _ in 0..digits {
        let digit = ((remaining / divisor) % 10) as u8;
        draw_glyph(buffer, b'0' + digit, (x, origin.1), scale);
        remaining %= divisor;
        divisor = (divisor / 10).max(1);
        x += (GLYPH_WIDTH + GLYPH_GAP) * scale;
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
        assert_eq!(clock_size(1), (47, 7));
        assert_eq!(clock_size(3), (141, 21));
    }

    #[test]
    fn digits_size_scales_with_the_scale() {
        assert_eq!(digits_size(4, 1), (23, 7));
        assert_eq!(digits_size(4, 2), (46, 14));
    }

    #[test]
    fn drawn_clock_stays_under_the_pixel_budget() {
        let mut buffer = MonoBuffer::new(PANEL_WIDTH, PANEL_HEIGHT);
        draw_time(&mut buffer, 10, 9, 30, centered_origin(), SCALE);

        let lit = buffer.lit_pixels();
        let total = buffer.total_pixels();
        assert!(lit > 0, "clock drew nothing");
        assert!(lit * 100 < total * 5, "{lit} of {total} lit exceeds 5%");
    }

    #[test]
    fn digits_draw_within_their_size() {
        let mut buffer = MonoBuffer::new(PANEL_WIDTH, PANEL_HEIGHT);
        let origin = (10u16, 10u16);
        draw_digits(&mut buffer, 3872, 4, origin, SCALE);

        let (width, height) = digits_size(4, SCALE);
        let mut outside_lit = 0;
        for y in 0..PANEL_HEIGHT {
            for x in 0..PANEL_WIDTH {
                let inside =
                    x >= origin.0 && x < origin.0 + width && y >= origin.1 && y < origin.1 + height;
                if !inside && buffer.is_lit(x, y) {
                    outside_lit += 1;
                }
            }
        }
        assert_eq!(outside_lit, 0);
        assert!(buffer.lit_pixels() > 0);
    }

    #[test]
    fn nothing_is_drawn_outside_the_clock_region() {
        let mut buffer = MonoBuffer::new(PANEL_WIDTH, PANEL_HEIGHT);
        let origin = centered_origin();
        draw_time(&mut buffer, 10, 9, 30, origin, SCALE);

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
