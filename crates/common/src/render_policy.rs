//! Runtime rendering policy.
//!
//! Drives UI behavior per [`Mode`](crate::Mode): pixel format, animation, and a
//! lit-pixel budget. Owned by `power`; read by `ui` and apps.

use crate::mode::Mode;

/// Pixel output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum PixelMode {
    /// 1-bit black and white.
    Mono1,
    /// 8-bit grayscale.
    Gray8,
}

/// How the UI should render for the current mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RenderPolicy {
    /// Pixel output format.
    pub pixel: PixelMode,
    /// Whether animations are allowed.
    pub is_animated: bool,
    /// Maximum percentage of lit pixels budgeted.
    pub pixel_budget_percent: u8,
}

impl RenderPolicy {
    /// Watch-mode policy: 1-bit, no animation, under 5% lit pixels.
    pub const WATCH: Self = Self {
        pixel: PixelMode::Mono1,
        is_animated: false,
        pixel_budget_percent: 5,
    };

    /// Desk-mode policy: 8-bit grayscale, animations, full pixel budget.
    pub const DESK: Self = Self {
        pixel: PixelMode::Gray8,
        is_animated: true,
        pixel_budget_percent: 100,
    };

    /// The policy for a runtime mode.
    pub const fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Watch => Self::WATCH,
            Mode::Desk => Self::DESK,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_is_monochrome_and_static() {
        let policy = RenderPolicy::for_mode(Mode::Watch);
        assert_eq!(policy.pixel, PixelMode::Mono1);
        assert!(!policy.is_animated);
        assert_eq!(policy.pixel_budget_percent, 5);
    }

    #[test]
    fn desk_is_grayscale_and_animated() {
        let policy = RenderPolicy::for_mode(Mode::Desk);
        assert_eq!(policy.pixel, PixelMode::Gray8);
        assert!(policy.is_animated);
    }
}
