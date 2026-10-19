//! Temporary first-light label.
//!
//! Paints one centered line through a surface, with no shell and no apps. The
//! firmware uses it to prove the display, surface, and flush path before the
//! shell boots.
// TODO(phase-6.3): remove once the shell renders the debug Home.

use crate::painter::Painter;
use crate::text::{draw_text, measure_uncached, TextAlign};
use crate::theme::Theme;

/// Clears the surface and draws one centered line.
pub fn draw_boot_label(painter: &mut Painter<'_>, text: &str, theme: &Theme) {
    let width = painter.width();
    let height = painter.height();
    painter.clear(theme.tones.background);
    if width == 0 || height == 0 {
        return;
    }

    let style = theme.type_scale.display;
    let metrics = measure_uncached(text, &style, width);
    let top = height.saturating_sub(metrics.height) / 2;
    draw_text(
        painter,
        text,
        &style,
        (0, top),
        width,
        TextAlign::Center,
        theme.tones.text,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{GraySurface, MonoSurface, Surface};
    use crate::test_support::mono_ppm;
    use crate::Theme;

    const WIDTH: u16 = 224;
    const HEIGHT: u16 = 160;

    #[test]
    fn watch_label_paints_on_mono() {
        let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
        let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            draw_boot_label(&mut painter, "WATCH", &Theme::WATCH);
        }
        assert_eq!(surface.pixel(0, 0), crate::Luma::BLACK);
        assert!(surface.lit_pixels() > 0);
        assert!(!mono_ppm(&surface).is_empty());
    }

    #[test]
    fn desk_label_paints_grayscale() {
        let mut data = vec![0u8; GraySurface::buffer_len(WIDTH, HEIGHT)];
        let mut surface = GraySurface::new(WIDTH, HEIGHT, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            draw_boot_label(&mut painter, "DESK", &Theme::DESK);
        }
        assert_eq!(surface.pixel(0, 0), crate::Luma::BLACK);
        assert!(surface.pixels().iter().any(|&value| value != 0));
        assert!(surface.pixels().iter().any(|&value| value != 255));
    }
}
