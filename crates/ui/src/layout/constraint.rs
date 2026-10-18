//! Integer sizes, constraints, length resolution, and insets.

/// The largest representable dimension, used as the unbounded sentinel.
pub const UNBOUNDED: u16 = u16::MAX;

/// A width and height in pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    /// Width in pixels.
    pub width: u16,
    /// Height in pixels.
    pub height: u16,
}

impl Size {
    /// A zero size.
    pub const ZERO: Self = Self::new(0, 0);

    /// Creates a size.
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    /// Whether either axis is zero.
    pub const fn is_empty(self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Minimum and maximum sizes passed down the layout tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Constraints {
    /// Smallest permitted size.
    pub min: Size,
    /// Largest permitted size.
    pub max: Size,
}

impl Constraints {
    /// Loose constraints: any size up to `max`.
    pub const fn loose(max: Size) -> Self {
        Self {
            min: Size::ZERO,
            max,
        }
    }

    /// Tight constraints: exactly `size`.
    pub const fn tight(size: Size) -> Self {
        Self {
            min: size,
            max: size,
        }
    }

    /// Unbounded constraints on both axes.
    pub const fn unbounded() -> Self {
        Self::loose(Size::new(UNBOUNDED, UNBOUNDED))
    }

    /// Clamps a size into this range.
    pub const fn constrain(self, size: Size) -> Size {
        let min_width = if self.min.width < self.max.width {
            self.min.width
        } else {
            self.max.width
        };
        let min_height = if self.min.height < self.max.height {
            self.min.height
        } else {
            self.max.height
        };
        Size::new(
            clamp(size.width, min_width, self.max.width),
            clamp(size.height, min_height, self.max.height),
        )
    }
}

const fn clamp(value: u16, min: u16, max: u16) -> u16 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

/// How one axis of a node is sized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Length {
    /// An exact pixel size.
    Fixed(u16),
    /// A share of the remaining space. Weights of zero act as one.
    Fill(u16),
    /// A percentage of the available space, clamped to 100.
    Percent(u8),
    /// The measured content size.
    Fit,
}

impl Length {
    /// Whether this length consumes remaining space.
    pub const fn is_fill(self) -> bool {
        matches!(self, Self::Fill(_))
    }

    /// The fill weight, with zero normalized to one.
    pub const fn weight(self) -> u16 {
        match self {
            Self::Fill(0) => 1,
            Self::Fill(weight) => weight,
            _ => 0,
        }
    }
}

/// Resolves a length against available space and a measured fit size.
pub(crate) const fn resolve(length: Length, available: u16, fit: u16) -> u16 {
    match length {
        Length::Fixed(value) => value,
        Length::Fit => fit,
        Length::Percent(percent) => {
            if available == UNBOUNDED {
                fit
            } else {
                percent_of(available, percent)
            }
        }
        Length::Fill(_) => {
            if available == UNBOUNDED {
                fit
            } else {
                available
            }
        }
    }
}

/// A percentage of a dimension, clamped to 100 percent.
pub(crate) const fn percent_of(available: u16, percent: u8) -> u16 {
    let percent = if percent > 100 { 100 } else { percent };
    ((available as u32 * percent as u32) / 100) as u16
}
