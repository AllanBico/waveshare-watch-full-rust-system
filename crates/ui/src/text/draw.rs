//! Text measurement and drawing.

use crate::luma::Luma;
use crate::painter::Painter;
use crate::theme::TextStyle;

use super::cache::TextCache;
use super::font::{font_asset, FontAsset, Glyph};
use super::layout::{layout_text, measure_layout, Line, TextMetrics};

/// Horizontal text alignment within the available width.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextAlign {
    /// Align to the start edge.
    Start,
    /// Center in the available width.
    Center,
    /// Align to the end edge.
    End,
}

/// Measures wrapped text, using the cache.
pub fn measure_text(
    text: &str,
    style: &TextStyle,
    max_width: u16,
    cache: &mut TextCache,
) -> TextMetrics {
    if let Some(metrics) = cache.get(text, style, max_width) {
        return metrics;
    }
    let metrics = measure_uncached(text, style, max_width);
    cache.insert(text, style, max_width, metrics);
    metrics
}

/// Measures wrapped text without the cache.
pub fn measure_uncached(text: &str, style: &TextStyle, max_width: u16) -> TextMetrics {
    let Some(asset) = font_asset(style.font, style.weight, style.size_px) else {
        return TextMetrics::ZERO;
    };
    let lines = layout_text(text, asset, max_width);
    measure_layout(&lines, style.line_height_px)
}

/// Draws wrapped text with its top-left at `origin`.
pub fn draw_text(
    painter: &mut Painter<'_>,
    text: &str,
    style: &TextStyle,
    origin: (u16, u16),
    max_width: u16,
    align: TextAlign,
    color: Luma,
) {
    let Some(asset) = font_asset(style.font, style.weight, style.size_px) else {
        return;
    };
    let lines = layout_text(text, asset, max_width);
    for (index, line) in lines.lines().iter().enumerate() {
        let line_top = origin
            .1
            .saturating_add(style.line_height_px.saturating_mul(index as u16));
        let x = aligned_x(origin.0, line, max_width, align);
        draw_line(painter, asset, line, x, line_top, color);
    }
}

fn aligned_x(origin_x: u16, line: &Line<'_>, max_width: u16, align: TextAlign) -> u16 {
    let available = if max_width == 0 {
        line.width
    } else {
        max_width
    };
    match align {
        TextAlign::Start => origin_x,
        TextAlign::Center => origin_x.saturating_add(available.saturating_sub(line.width) / 2),
        TextAlign::End => origin_x.saturating_add(available.saturating_sub(line.width)),
    }
}

fn draw_line(
    painter: &mut Painter<'_>,
    asset: &FontAsset,
    line: &Line<'_>,
    x: u16,
    top: u16,
    color: Luma,
) {
    let baseline = i32::from(top) + i32::from(asset.ascent);
    let mut pen = i32::from(x);
    for character in line.text.chars() {
        let Some(glyph) = asset.glyph_or_fallback(character) else {
            continue;
        };
        draw_glyph(painter, asset, glyph, pen, baseline, color);
        pen += i32::from(glyph.advance);
    }
}

fn draw_glyph(
    painter: &mut Painter<'_>,
    asset: &FontAsset,
    glyph: &Glyph,
    pen: i32,
    baseline: i32,
    color: Luma,
) {
    let mask = asset.mask(glyph);
    let glyph_top = baseline - (i32::from(glyph.ymin) + i32::from(glyph.height));
    for row in 0..glyph.height {
        for col in 0..glyph.width {
            let index = usize::from(row) * usize::from(glyph.width) + usize::from(col);
            let Some(coverage) = nibble(mask, index) else {
                continue;
            };
            let x = pen + i32::from(glyph.xmin) + i32::from(col);
            let y = glyph_top + i32::from(row);
            let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
                continue;
            };
            painter.blend_pixel(x, y, color, coverage);
        }
    }
}

fn nibble(mask: &[u8], index: usize) -> Option<u8> {
    let byte = *mask.get(index / 2)?;
    Some(if index.is_multiple_of(2) {
        byte >> 4
    } else {
        byte & 0x0F
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::MonoSurface;
    use crate::test_support::{fnv1a, mono_ppm};
    use crate::theme::Theme;

    fn with_surface(width: u16, height: u16, run: impl FnOnce(&mut Painter<'_>)) -> u32 {
        let mut data = vec![0u8; MonoSurface::buffer_len(width, height)];
        let mut surface = MonoSurface::new(width, height, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            run(&mut painter);
        }
        surface.lit_pixels()
    }

    #[test]
    fn text_lights_pixels() {
        let style = Theme::WATCH.type_scale.title;
        let lit = with_surface(128, 48, |painter| {
            draw_text(
                painter,
                "Hi",
                &style,
                (0, 0),
                0,
                TextAlign::Start,
                Luma::WHITE,
            );
        });
        assert!(lit > 0);
    }

    #[test]
    fn measurement_is_cached() {
        let style = Theme::WATCH.type_scale.body;
        let mut cache = TextCache::new();
        let first = measure_text("Hello", &style, 100, &mut cache);
        assert!(cache.len() == 1);
        let second = measure_text("Hello", &style, 100, &mut cache);
        assert_eq!(first, second);
    }

    #[test]
    fn text_golden_hash_is_stable() {
        let style = Theme::WATCH.type_scale.numeric;
        let mut data = vec![0u8; MonoSurface::buffer_len(180, 96)];
        let mut surface = MonoSurface::new(180, 96, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            draw_text(
                &mut painter,
                "12:34",
                &style,
                (4, 4),
                0,
                TextAlign::Start,
                Luma::WHITE,
            );
            crate::draw_icon(
                &mut painter,
                crate::IconId::Clock,
                24,
                (148, 4),
                Luma::WHITE,
            );
        }
        assert!(surface.lit_pixels() > 0);
        assert_eq!(fnv1a(&mono_ppm(&surface)), 0x4890_e23d_2957_2f02);
    }
}
