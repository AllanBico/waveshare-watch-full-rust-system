//! Typed design tokens shared by the watch and desk themes.

use crate::luma::Luma;

/// Semantic luminance roles. Components read roles, never raw values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tones {
    /// Root background.
    pub background: Luma,
    /// Default surface.
    pub surface: Luma,
    /// Raised or focused surface.
    pub surface_raised: Luma,
    /// Primary text and icons.
    pub text: Luma,
    /// Secondary text and icons.
    pub text_muted: Luma,
    /// Dividers and outlines.
    pub border: Luma,
    /// Focus ring.
    pub focus: Luma,
    /// Error accent.
    pub error: Luma,
    /// Selected item surface.
    pub selected: Luma,
    /// Pressed item surface.
    pub pressed: Luma,
    /// Disabled content.
    pub disabled: Luma,
}

/// A compiled-in font face.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FontId {
    /// Proportional interface face.
    Ui = 0,
    /// Tabular numerals for clock and data.
    Numeric = 1,
}

/// A font weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Weight {
    /// Regular.
    Regular = 0,
    /// Medium.
    Medium = 1,
    /// Semibold.
    Semibold = 2,
}

/// A named text style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextStyle {
    /// Font face.
    pub font: FontId,
    /// Cap size in pixels.
    pub size_px: u16,
    /// Line height in pixels.
    pub line_height_px: u16,
    /// Font weight.
    pub weight: Weight,
}

/// Typography roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Typography {
    /// Hero text, such as the home clock.
    pub display: TextStyle,
    /// Screen title.
    pub title: TextStyle,
    /// Primary body text.
    pub body: TextStyle,
    /// Secondary text.
    pub caption: TextStyle,
    /// Control label.
    pub label: TextStyle,
    /// Tabular numerals.
    pub numeric: TextStyle,
}

impl Typography {
    /// The shared type scale. The panel has one physical size in both modes.
    pub const BASE: Self = Self {
        display: TextStyle {
            font: FontId::Ui,
            size_px: 48,
            line_height_px: 56,
            weight: Weight::Regular,
        },
        title: TextStyle {
            font: FontId::Ui,
            size_px: 36,
            line_height_px: 44,
            weight: Weight::Medium,
        },
        body: TextStyle {
            font: FontId::Ui,
            size_px: 28,
            line_height_px: 36,
            weight: Weight::Regular,
        },
        caption: TextStyle {
            font: FontId::Ui,
            size_px: 22,
            line_height_px: 28,
            weight: Weight::Regular,
        },
        label: TextStyle {
            font: FontId::Ui,
            size_px: 24,
            line_height_px: 30,
            weight: Weight::Medium,
        },
        numeric: TextStyle {
            font: FontId::Numeric,
            size_px: 72,
            line_height_px: 84,
            weight: Weight::Medium,
        },
    };
}

/// Icon sizes in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconSizes {
    /// Small icon.
    pub sm: u16,
    /// Default icon.
    pub md: u16,
    /// Large icon.
    pub lg: u16,
    /// Extra-large icon.
    pub xl: u16,
}

impl IconSizes {
    /// The shared icon scale.
    pub const DEFAULT: Self = Self {
        sm: 24,
        md: 32,
        lg: 40,
        xl: 48,
    };
}

/// Spacing scale in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spacing {
    /// 2 px.
    pub xxs: u16,
    /// 4 px.
    pub xs: u16,
    /// 8 px.
    pub sm: u16,
    /// 12 px.
    pub md: u16,
    /// 16 px.
    pub lg: u16,
    /// 24 px.
    pub xl: u16,
    /// 32 px.
    pub xxl: u16,
}

impl Spacing {
    /// The shared spacing scale.
    pub const SCALE: Self = Self {
        xxs: 2,
        xs: 4,
        sm: 8,
        md: 12,
        lg: 16,
        xl: 24,
        xxl: 32,
    };
}

/// Corner radii in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Radii {
    /// Small radius.
    pub sm: u16,
    /// Medium radius.
    pub md: u16,
    /// Large radius.
    pub lg: u16,
    /// Fully rounded.
    pub pill: u16,
}

impl Radii {
    /// The shared radius scale.
    pub const DEFAULT: Self = Self {
        sm: 8,
        md: 12,
        lg: 16,
        pill: 255,
    };
}

/// Border widths in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Borders {
    /// Hairline.
    pub thin: u16,
    /// Emphasized border.
    pub thick: u16,
}

impl Borders {
    /// The shared border scale.
    pub const DEFAULT: Self = Self { thin: 1, thick: 2 };
}

/// Motion durations in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Motion {
    /// Short feedback transition.
    pub fast_ms: u16,
    /// Standard transition.
    pub normal_ms: u16,
    /// Larger overlay transition.
    pub slow_ms: u16,
}

impl Motion {
    /// No motion; used in watch mode.
    pub const NONE: Self = Self {
        fast_ms: 0,
        normal_ms: 0,
        slow_ms: 0,
    };

    /// Desk-mode motion.
    pub const DESK: Self = Self {
        fast_ms: 120,
        normal_ms: 200,
        slow_ms: 300,
    };
}
