//! Touch input vocabulary.

/// Maximum simultaneous touch points supported by the FT3168.
///
/// The FT3168 is a self-capacitance controller that reports either one point
/// with gestures or two points; see the datasheet feature list.
pub const MAX_TOUCH_POINTS: usize = 2;

/// One active touch point in panel coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TouchPoint {
    /// Controller-assigned touch identifier.
    pub id: u8,
    /// Horizontal coordinate in pixels.
    pub x: u16,
    /// Vertical coordinate in pixels.
    pub y: u16,
}

impl TouchPoint {
    /// Creates a touch point.
    pub const fn new(id: u8, x: u16, y: u16) -> Self {
        Self { id, x, y }
    }
}

/// A consistent snapshot of the active touch points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TouchReport {
    count: u8,
    points: [TouchPoint; MAX_TOUCH_POINTS],
}

impl TouchReport {
    /// An empty report, meaning no fingers are down.
    pub const fn empty() -> Self {
        Self {
            count: 0,
            points: [TouchPoint::new(0, 0, 0); MAX_TOUCH_POINTS],
        }
    }

    /// Builds a report from up to [`MAX_TOUCH_POINTS`] points.
    pub fn new(points: &[TouchPoint]) -> Self {
        let count = points.len().min(MAX_TOUCH_POINTS);
        let mut buffer = [TouchPoint::new(0, 0, 0); MAX_TOUCH_POINTS];
        buffer[..count].copy_from_slice(&points[..count]);
        Self {
            count: count as u8,
            points: buffer,
        }
    }

    /// The number of active touch points.
    pub const fn count(&self) -> u8 {
        self.count
    }

    /// Whether no fingers are down.
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// The active points, in controller report order.
    pub fn points(&self) -> &[TouchPoint] {
        &self.points[..self.count as usize]
    }

    /// One active point by index.
    pub fn get(&self, index: usize) -> Option<&TouchPoint> {
        self.points().get(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_has_no_points() {
        let report = TouchReport::empty();
        assert_eq!(report.count(), 0);
        assert!(report.is_empty());
        assert!(report.points().is_empty());
        assert!(report.get(0).is_none());
    }

    #[test]
    fn new_copies_points_in_order() {
        let points = [TouchPoint::new(1, 10, 20), TouchPoint::new(2, 30, 40)];
        let report = TouchReport::new(&points);
        assert_eq!(report.count(), 2);
        assert!(!report.is_empty());
        assert_eq!(report.points(), &points);
        assert_eq!(report.get(1), Some(&points[1]));
    }

    #[test]
    fn new_clamps_to_the_hardware_limit() {
        let points = [
            TouchPoint::new(0, 1, 2),
            TouchPoint::new(1, 3, 4),
            TouchPoint::new(2, 5, 6),
        ];
        let report = TouchReport::new(&points);
        assert_eq!(report.count(), MAX_TOUCH_POINTS as u8);
        assert_eq!(report.points(), &points[..MAX_TOUCH_POINTS]);
    }
}
