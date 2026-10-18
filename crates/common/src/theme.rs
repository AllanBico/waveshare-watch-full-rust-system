//! Theme identifiers shared by settings and the UI.
//!
//! The concrete palette stays in `ui`; this is the stable persisted name.

/// A selectable theme variant.
///
/// Every variant is valid in both pixel modes, but a 1-bit surface can only
/// render one palette. The UI resolves the variant against the active pixel
/// mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum ThemeId {
    /// The standard palette for the pixel mode.
    Default = 0,
    /// Maximum separation between text and surfaces.
    HighContrast = 1,
}

impl ThemeId {
    /// Every theme variant, in stable index order.
    pub const ALL: [Self; 2] = [Self::Default, Self::HighContrast];

    /// Stable dense index used by settings storage.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Resolves a stored index.
    pub const fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(Self::Default),
            1 => Some(Self::HighContrast),
            _ => None,
        }
    }

    /// The next variant, wrapping.
    pub const fn next(self) -> Self {
        match self {
            Self::Default => Self::HighContrast,
            Self::HighContrast => Self::Default,
        }
    }

    /// Short UI label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::HighContrast => "High Contrast",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_roundtrip() {
        for theme in ThemeId::ALL {
            assert_eq!(ThemeId::from_index(theme.index()), Some(theme));
        }
        assert_eq!(ThemeId::from_index(2), None);
    }

    #[test]
    fn next_wraps() {
        assert_eq!(ThemeId::Default.next(), ThemeId::HighContrast);
        assert_eq!(ThemeId::HighContrast.next(), ThemeId::Default);
    }
}
