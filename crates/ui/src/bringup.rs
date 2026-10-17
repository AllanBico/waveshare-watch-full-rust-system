//! Temporary display bring-up pattern.
//!
//! This keeps on-device verification alive between the old viewer and the first
//! real screens. It draws shapes only, so it exercises surfaces and the painter
//! without fonts or assets.
// TODO(phase-5): remove once Home renders through the shell.

use common::Region;

use crate::luma::Luma;
use crate::painter::Painter;

/// Draws a low-pixel test pattern in the current surface.
pub fn draw_bringup(painter: &mut Painter<'_>) {
    let width = painter.width();
    let height = painter.height();
    if width == 0 || height == 0 {
        return;
    }
    painter.clear(Luma::BLACK);

    let margin = width.min(height) / 16;
    let frame = Region::new(
        margin,
        margin,
        width.saturating_sub(margin * 2),
        height.saturating_sub(margin * 2),
    );
    painter.stroke_rect(frame, 2, Luma::WHITE);

    let box_width = width / 2;
    let box_height = height / 4;
    let outer = Region::new(
        (width - box_width) / 2,
        (height - box_height) / 2,
        box_width,
        box_height,
    );
    painter.fill_rounded_rect(outer, 24, Luma::WHITE);
    let inset = 3;
    painter.fill_rounded_rect(
        Region::new(
            outer.x + inset,
            outer.y + inset,
            outer.width.saturating_sub(inset * 2),
            outer.height.saturating_sub(inset * 2),
        ),
        21,
        Luma::BLACK,
    );

    painter.line(0, height / 2, width - 1, height / 2, Luma::WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{MonoSurface, Surface};
    use crate::test_support::{fnv1a, mono_ppm};

    const WIDTH: u16 = 96;
    const HEIGHT: u16 = 96;

    fn render() -> u64 {
        let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
        let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            draw_bringup(&mut painter);
        }
        assert_eq!(surface.pixel(0, 0), Luma::BLACK);
        assert!(surface.lit_pixels() > 0);
        fnv1a(&mono_ppm(&surface))
    }

    #[test]
    fn bringup_golden_hash_is_stable() {
        assert_eq!(render(), 0xc27a_e187_e428_c737);
    }
}
