//! Rounded panel geometry and shape-aware layout spans.

use crate::layout::Insets;

/// The panel silhouette and the spacing tokens derived from it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScreenShape {
    /// Physical corner radius in pixels.
    pub corner_radius: u16,
    /// Optical inset for protected content.
    pub content_inset: u16,
    /// Horizontal inset for crown content.
    pub crown_inset: u16,
}

impl ScreenShape {
    /// A rectangular screen with no corner treatment.
    pub const NONE: Self = Self {
        corner_radius: 0,
        content_inset: 0,
        crown_inset: 0,
    };

    /// The calibrated 410 by 502 AMOLED silhouette.
    pub const PANEL: Self = Self {
        corner_radius: 112,
        content_inset: 12,
        crown_inset: 72,
    };

    /// Whether the shape has any corner cutout.
    pub const fn has_corners(self) -> bool {
        self.corner_radius > 0
    }

    /// The vertical inset at which a content box clears the corner arcs.
    pub fn vertical_inset(self) -> u16 {
        if self.corner_radius == 0 {
            return self.content_inset;
        }
        let radius = u32::from(self.corner_radius);
        let inset = u32::from(self.content_inset.min(self.corner_radius));
        let inner = radius - inset;
        let squared = radius * radius - inner * inner;
        (radius - integer_sqrt(squared)) as u16
    }

    /// Insets for a full-height content column below the crown.
    pub fn content_insets(self) -> Insets {
        let vertical = self.vertical_inset();
        Insets::new(vertical, self.content_inset, vertical, self.content_inset)
    }

    /// Horizontal insets for content inside the crown.
    pub fn crown_insets(self) -> Insets {
        Insets::symmetric(self.crown_inset, 0)
    }

    /// The bottom content run-out.
    pub fn bottom_inset(self) -> u16 {
        self.vertical_inset()
    }

    /// The horizontal span inside the physical shape for one row.
    pub fn row_span(self, y: u16, width: u16, height: u16) -> (u16, u16) {
        span(y, width, height, self.corner_radius, 0)
    }

    /// The horizontal span inside the content-safe shape for one row.
    pub fn safe_row_span(self, y: u16, width: u16, height: u16) -> (u16, u16) {
        span(y, width, height, self.corner_radius, self.content_inset)
    }

    /// Whether a point lies inside the physical shape.
    pub fn contains(self, x: u16, y: u16, width: u16, height: u16) -> bool {
        if !self.has_corners() {
            return true;
        }
        let (left, right) = self.row_span(y, width, height);
        x >= left && x < right
    }
}

fn span(y: u16, width: u16, height: u16, radius: u16, inset: u16) -> (u16, u16) {
    let doubled = inset.saturating_mul(2);
    let left_edge = inset.min(width);
    let top_edge = inset.min(height);
    let inner_width = width.saturating_sub(doubled);
    let inner_height = height.saturating_sub(doubled);
    if inner_width == 0 || inner_height == 0 {
        return (0, 0);
    }
    let bottom_edge = top_edge.saturating_add(inner_height).saturating_sub(1);
    if y < top_edge || y > bottom_edge {
        return (0, 0);
    }
    let radius = radius
        .saturating_sub(inset)
        .min(inner_width.min(inner_height) / 2);
    let mut left = left_edge;
    let mut right = left_edge.saturating_add(inner_width);
    if radius > 0 {
        if y < top_edge.saturating_add(radius) {
            let dy = top_edge + radius - y;
            let dx = arc_dx(radius, dy);
            left = (left_edge + radius).saturating_sub(dx);
            right = right.saturating_sub(radius).saturating_add(dx);
        } else if y > bottom_edge.saturating_sub(radius) {
            let dy = y - (bottom_edge - radius);
            let dx = arc_dx(radius, dy);
            left = (left_edge + radius).saturating_sub(dx);
            right = right.saturating_sub(radius).saturating_add(dx);
        }
    }
    (left, right.min(left_edge.saturating_add(inner_width)))
}

fn arc_dx(radius: u16, dy: u16) -> u16 {
    let squared = u32::from(radius) * u32::from(radius);
    let offset = u32::from(dy.min(radius));
    integer_sqrt(squared - offset * offset) as u16
}

pub(crate) fn integer_sqrt(value: u32) -> u32 {
    let mut result = 0u32;
    let mut bit = 1u32 << 30;
    let mut remainder = value;
    while bit > remainder {
        bit >>= 2;
    }
    while bit != 0 {
        if remainder >= result + bit {
            remainder -= result + bit;
            result = (result >> 1) + bit;
        } else {
            result >>= 1;
        }
        bit >>= 2;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: u16 = 410;
    const HEIGHT: u16 = 502;

    #[test]
    fn panel_vertical_inset_clears_the_corners() {
        assert_eq!(ScreenShape::PANEL.vertical_inset(), 62);
        let insets = ScreenShape::PANEL.content_insets();
        assert_eq!(insets.top, 62);
        assert_eq!(insets.bottom, 62);
        assert_eq!(insets.left, 12);
        assert_eq!(insets.right, 12);
    }

    #[test]
    fn top_row_is_inset_by_the_radius() {
        assert_eq!(ScreenShape::PANEL.row_span(0, WIDTH, HEIGHT), (112, 298));
        assert_eq!(
            ScreenShape::PANEL.row_span(HEIGHT / 2, WIDTH, HEIGHT),
            (0, WIDTH)
        );
    }

    #[test]
    fn content_corners_are_safe() {
        let shape = ScreenShape::PANEL;
        let insets = shape.content_insets();
        let left = insets.left;
        let top = insets.top;
        let right = WIDTH.saturating_sub(insets.right);
        let bottom = HEIGHT.saturating_sub(insets.bottom);
        assert!(shape.contains(left, top, WIDTH, HEIGHT));
        assert!(shape.contains(right - 1, top, WIDTH, HEIGHT));
        assert!(shape.contains(left, bottom - 1, WIDTH, HEIGHT));
        assert!(shape.contains(right - 1, bottom - 1, WIDTH, HEIGHT));
    }

    #[test]
    fn empty_shape_keeps_the_full_screen() {
        let shape = ScreenShape::NONE;
        assert!(shape.contains(0, 0, WIDTH, HEIGHT));
        assert_eq!(shape.row_span(0, WIDTH, HEIGHT), (0, WIDTH));
        assert_eq!(shape.row_span(HEIGHT - 1, WIDTH, HEIGHT), (0, WIDTH));
    }
}
