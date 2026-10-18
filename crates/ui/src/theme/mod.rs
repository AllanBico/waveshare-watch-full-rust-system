//! Compiled-in monochrome themes.
//!
//! Both themes use a black background. Watch maps every role to one bit; desk
//! adds tonal steps for surfaces, secondary text, and antialiasing.

mod tokens;

pub use tokens::{
    Borders, FontId, IconSizes, Motion, Radii, Spacing, TextStyle, Tones, Typography, Weight,
};

use common::PixelMode;

use crate::luma::Luma;
use crate::shape::ScreenShape;

/// Every UI token needed to render a screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    /// Semantic luminance roles.
    pub tones: Tones,
    /// Type scale.
    pub type_scale: Typography,
    /// Icon sizes.
    pub icons: IconSizes,
    /// Spacing scale.
    pub spacing: Spacing,
    /// Corner radii.
    pub radii: Radii,
    /// Border widths.
    pub borders: Borders,
    /// Motion durations.
    pub motion: Motion,
    /// Minimum touch target in pixels.
    pub min_touch_px: u16,
    /// Pixel format this theme targets.
    pub pixel_mode: PixelMode,
    /// Physical panel silhouette and shape-derived insets.
    pub shape: ScreenShape,
}

impl Theme {
    /// Watch theme: pure black and white, no motion.
    pub const WATCH: Self = Self {
        tones: Tones {
            background: Luma::BLACK,
            surface: Luma::BLACK,
            surface_raised: Luma::BLACK,
            text: Luma::WHITE,
            text_muted: Luma::WHITE,
            border: Luma::WHITE,
            focus: Luma::WHITE,
            error: Luma::WHITE,
            selected: Luma::WHITE,
            pressed: Luma::WHITE,
            disabled: Luma::WHITE,
        },
        type_scale: Typography::BASE,
        icons: IconSizes::DEFAULT,
        spacing: Spacing::SCALE,
        radii: Radii::DEFAULT,
        borders: Borders::DEFAULT,
        motion: Motion::NONE,
        min_touch_px: 64,
        pixel_mode: PixelMode::Mono1,
        shape: ScreenShape::PANEL,
    };

    /// Desk theme: black background with dark gray surfaces and antialiasing.
    pub const DESK: Self = Self {
        tones: Tones {
            background: Luma::BLACK,
            surface: Luma::new(10),
            surface_raised: Luma::new(20),
            text: Luma::new(240),
            text_muted: Luma::new(150),
            border: Luma::new(55),
            focus: Luma::WHITE,
            error: Luma::new(255),
            selected: Luma::new(210),
            pressed: Luma::new(180),
            disabled: Luma::new(90),
        },
        type_scale: Typography::BASE,
        icons: IconSizes::DEFAULT,
        spacing: Spacing::SCALE,
        radii: Radii::DEFAULT,
        borders: Borders::DEFAULT,
        motion: Motion::DESK,
        min_touch_px: 64,
        pixel_mode: PixelMode::Gray8,
        shape: ScreenShape::PANEL,
    };

    /// The theme for a pixel format.
    pub const fn for_pixel_mode(pixel: PixelMode) -> &'static Self {
        match pixel {
            PixelMode::Mono1 => &Self::WATCH,
            _ => &Self::DESK,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_is_pure_black_and_white() {
        let theme = Theme::for_pixel_mode(PixelMode::Mono1);
        assert_eq!(theme.tones.background, Luma::BLACK);
        assert_eq!(theme.tones.surface, Luma::BLACK);
        assert_eq!(theme.tones.text, Luma::WHITE);
        assert_eq!(theme.tones.focus, Luma::WHITE);
        assert_eq!(theme.tones.error, Luma::WHITE);
        assert_eq!(theme.tones.selected, Luma::WHITE);
        assert_eq!(theme.tones.pressed, Luma::WHITE);
        assert_eq!(theme.pixel_mode, PixelMode::Mono1);
        assert_eq!(theme.motion, Motion::NONE);
    }

    #[test]
    fn desk_has_tonal_separation() {
        let theme = Theme::for_pixel_mode(PixelMode::Gray8);
        assert_eq!(theme.tones.background, Luma::BLACK);
        assert!(theme.tones.surface > theme.tones.background);
        assert!(theme.tones.surface_raised > theme.tones.surface);
        assert!(theme.tones.text > theme.tones.text_muted);
        assert!(theme.tones.selected > theme.tones.surface_raised);
        assert!(theme.tones.pressed > theme.tones.surface_raised);
        assert_eq!(theme.pixel_mode, PixelMode::Gray8);
        assert!(theme.motion.normal_ms > 0);
    }

    #[test]
    fn touch_targets_clear_the_minimum() {
        const { assert!(Theme::WATCH.min_touch_px >= 64) };
        const { assert!(Theme::DESK.min_touch_px >= 64) };
    }

    #[test]
    fn type_roles_are_ordered() {
        let type_scale = Theme::WATCH.type_scale;
        assert!(type_scale.display.size_px > type_scale.title.size_px);
        assert!(type_scale.title.size_px > type_scale.body.size_px);
        assert!(type_scale.body.size_px > type_scale.caption.size_px);
        assert_eq!(type_scale.numeric.font, FontId::Numeric);
    }
}
