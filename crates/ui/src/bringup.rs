//! Temporary display bring-up pattern.
//!
//! This keeps on-device verification alive between the old viewer and the first
//! real screens. It exercises surfaces, the painter, compiled fonts, and icons.
// TODO(phase-5): remove once Home renders through the shell.

use common::Region;

use crate::icon::draw_icon;
use crate::luma::Luma;
use crate::painter::Painter;
use crate::text::{draw_text, measure_uncached, TextAlign};
use crate::theme::Theme;
use crate::IconId;

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

    let theme = Theme::WATCH;
    let title = theme.type_scale.title;
    let caption = theme.type_scale.caption;
    let title_y = frame.y + theme.spacing.lg;
    let metrics = measure_uncached("UI 5.2", &title, 0);
    draw_text(
        painter,
        "UI 5.2",
        &title,
        (
            frame.x + (frame.width.saturating_sub(metrics.width)) / 2,
            title_y,
        ),
        0,
        TextAlign::Start,
        Luma::WHITE,
    );

    let icon_size = theme.icons.md;
    draw_icon(
        painter,
        IconId::Clock,
        icon_size,
        (
            frame.x + (frame.width.saturating_sub(icon_size)) / 2,
            frame.y + frame.height / 3,
        ),
        Luma::WHITE,
    );

    let caption_metrics = measure_uncached("SORA + PHOSPHOR", &caption, frame.width);
    draw_text(
        painter,
        "SORA + PHOSPHOR",
        &caption,
        (
            frame.x + (frame.width.saturating_sub(caption_metrics.width)) / 2,
            frame.y + frame.height - caption_metrics.height - theme.spacing.lg,
        ),
        frame.width,
        TextAlign::Start,
        Luma::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::{MonoSurface, Surface};
    use crate::test_support::{fnv1a, mono_ppm};

    const WIDTH: u16 = 224;
    const HEIGHT: u16 = 160;

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
        assert_eq!(render(), 0xd364_8fda_c7fc_ec29);
    }
}
