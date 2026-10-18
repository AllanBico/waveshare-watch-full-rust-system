//! Compiled-in font lookup.

use crate::assets::FONT_ASSETS;
use crate::theme::{FontId, Weight};

/// One rasterized glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    /// Unicode scalar value.
    pub codepoint: u32,
    /// Mask width in pixels.
    pub width: u16,
    /// Mask height in pixels.
    pub height: u16,
    /// Left bearing from the pen position.
    pub xmin: i16,
    /// Bottom bearing from the baseline.
    pub ymin: i16,
    /// Horizontal advance.
    pub advance: u16,
    /// Mask byte offset.
    pub offset: u32,
    /// Mask byte length.
    pub len: u32,
}

/// One compiled-in font size.
#[derive(Debug, Clone, Copy)]
pub struct FontAsset {
    /// Font face.
    pub font_id: FontId,
    /// Font weight.
    pub weight: Weight,
    /// Pixel size.
    pub size_px: u16,
    /// Distance from the baseline to the top.
    pub ascent: i16,
    /// Distance from the baseline to the bottom.
    pub descent: i16,
    /// Extra gap between lines.
    pub line_gap: i16,
    /// Glyphs sorted by codepoint.
    pub glyphs: &'static [Glyph],
    /// Packed 4-bit coverage masks.
    pub masks: &'static [u8],
}

impl FontAsset {
    /// Finds the glyph for a character.
    pub fn glyph(&self, character: char) -> Option<&Glyph> {
        let key = character as u32;
        let index = self
            .glyphs
            .binary_search_by_key(&key, |glyph| glyph.codepoint)
            .ok()?;
        self.glyphs.get(index)
    }

    /// The glyph for a character, falling back to `?`.
    pub fn glyph_or_fallback(&self, character: char) -> Option<&Glyph> {
        self.glyph(character).or_else(|| self.glyph('?'))
    }

    /// The advance for a character.
    pub fn advance(&self, character: char) -> u16 {
        self.glyph(character)
            .or_else(|| self.glyph('?'))
            .or_else(|| self.glyph(' '))
            .map_or(0, |glyph| glyph.advance)
    }

    /// The packed mask for a glyph.
    pub fn mask(&self, glyph: &Glyph) -> &[u8] {
        let start = glyph.offset as usize;
        let Some(end) = start.checked_add(glyph.len as usize) else {
            return &[];
        };
        self.masks.get(start..end).unwrap_or(&[])
    }
}

/// Finds an exact font size.
pub fn font_asset(font: FontId, weight: Weight, size_px: u16) -> Option<&'static FontAsset> {
    FONT_ASSETS
        .iter()
        .find(|asset| asset.font_id == font && asset.weight == weight && asset.size_px == size_px)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn every_theme_style_has_an_asset() {
        let scale = Theme::WATCH.type_scale;
        for style in [
            scale.display,
            scale.title,
            scale.body,
            scale.caption,
            scale.label,
            scale.numeric,
        ] {
            assert!(
                font_asset(style.font, style.weight, style.size_px).is_some(),
                "missing asset for {style:?}"
            );
        }
    }

    #[test]
    fn glyphs_and_masks_are_present() {
        let asset = font_asset(FontId::Ui, Weight::Regular, 22).unwrap();
        let glyph = asset.glyph('A').unwrap();
        assert_eq!(glyph.codepoint, u32::from('A'));
        assert!(!asset.mask(glyph).is_empty());
        assert!(asset.glyph('\u{1F600}').is_none());
    }
}
