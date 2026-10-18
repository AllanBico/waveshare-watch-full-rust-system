//! Pure edge-gesture recognition with slop, threshold, and velocity.

use common::TouchPoint;

/// Gesture recognition thresholds in pixels and milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GestureConfig {
    /// Movement below this is still a tap.
    pub slop_px: u16,
    /// Vertical distance that qualifies a swipe.
    pub threshold_px: u16,
    /// Vertical speed that qualifies a swipe.
    pub velocity_px_per_s: u16,
    /// Distance from the top or bottom edge that starts an edge gesture.
    pub edge_margin_px: u16,
    /// Maximum press duration for a tap.
    pub tap_max_ms: u16,
}

impl GestureConfig {
    /// Defaults tuned for the 410 by 502 panel.
    pub const DEFAULT: Self = Self {
        slop_px: 12,
        threshold_px: 96,
        velocity_px_per_s: 500,
        edge_margin_px: 32,
        tap_max_ms: 300,
    };
}

impl Default for GestureConfig {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Where a gesture began.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeOrigin {
    /// Started inside the top edge margin.
    TopEdge,
    /// Started inside the bottom edge margin.
    BottomEdge,
    /// Started in the content area.
    Content,
}

/// The dominant swipe direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    /// Toward smaller y coordinates.
    Up,
    /// Toward larger y coordinates.
    Down,
    /// Toward smaller x coordinates.
    Left,
    /// Toward larger x coordinates.
    Right,
}

/// One recognized swipe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swipe {
    /// Where the touch started.
    pub origin: SwipeOrigin,
    /// The dominant direction.
    pub direction: SwipeDirection,
    /// Vertical travel in pixels.
    pub distance_px: u16,
    /// Signed vertical velocity in pixels per second.
    pub velocity_px_per_s: i32,
}

/// The result of one completed touch sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GestureOutcome {
    /// Nothing has completed yet.
    None,
    /// A short press that did not move past slop.
    Tap {
        /// Final horizontal position.
        x: u16,
        /// Final vertical position.
        y: u16,
    },
    /// A qualifying swipe.
    Swipe(Swipe),
    /// The touch was rejected or cancelled.
    Cancelled,
}

/// The recognition phase of the current touch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GesturePhase {
    /// No touch is tracked.
    Idle,
    /// A touch is down but has not cleared slop.
    Pressed,
    /// The touch is moving vertically.
    Tracking,
    /// The touch exceeded slop on the wrong axis.
    Cancelled,
}

/// A pure edge-gesture state machine.
pub struct EdgeGesture {
    config: GestureConfig,
    phase: GesturePhase,
    start: TouchPoint,
    last: TouchPoint,
    start_ms: u64,
    origin: SwipeOrigin,
}

impl EdgeGesture {
    /// Creates an idle recognizer.
    pub const fn new(config: GestureConfig) -> Self {
        Self {
            config,
            phase: GesturePhase::Idle,
            start: TouchPoint::new(0, 0, 0),
            last: TouchPoint::new(0, 0, 0),
            start_ms: 0,
            origin: SwipeOrigin::Content,
        }
    }

    /// The configured thresholds.
    pub const fn config(&self) -> GestureConfig {
        self.config
    }

    /// The current recognition phase.
    pub const fn phase(&self) -> GesturePhase {
        self.phase
    }

    /// Starts tracking a touch.
    pub fn begin(&mut self, point: TouchPoint, now_ms: u64, viewport_height: u16) {
        self.start = point;
        self.last = point;
        self.start_ms = now_ms;
        self.origin = edge_origin(point.y, viewport_height, self.config.edge_margin_px);
        self.phase = GesturePhase::Pressed;
    }

    /// Feeds a movement sample.
    pub fn update(&mut self, point: TouchPoint, now_ms: u64) {
        let _ = now_ms;
        if !matches!(self.phase, GesturePhase::Pressed | GesturePhase::Tracking) {
            return;
        }
        self.last = point;
        let dx = point.x.abs_diff(self.start.x);
        let dy = point.y.abs_diff(self.start.y);
        if dx > self.config.slop_px && dx > dy {
            self.phase = GesturePhase::Cancelled;
        } else if dx > self.config.slop_px || dy > self.config.slop_px {
            self.phase = GesturePhase::Tracking;
        }
    }

    /// Finishes the touch and classifies it.
    pub fn end(&mut self, now_ms: u64) -> GestureOutcome {
        let phase = core::mem::replace(&mut self.phase, GesturePhase::Idle);
        match phase {
            GesturePhase::Idle => GestureOutcome::None,
            GesturePhase::Cancelled => GestureOutcome::Cancelled,
            GesturePhase::Pressed | GesturePhase::Tracking => self.classify(now_ms),
        }
    }

    /// Drops the current touch without producing a result.
    pub fn cancel(&mut self) {
        self.phase = GesturePhase::Idle;
    }

    fn classify(&self, now_ms: u64) -> GestureOutcome {
        let dx = i32::from(self.last.x) - i32::from(self.start.x);
        let dy = i32::from(self.last.y) - i32::from(self.start.y);
        let duration_ms = now_ms.saturating_sub(self.start_ms).max(1);
        let slop = i32::from(self.config.slop_px);

        if dx.abs() <= slop && dy.abs() <= slop && duration_ms <= u64::from(self.config.tap_max_ms)
        {
            return GestureOutcome::Tap {
                x: self.last.x,
                y: self.last.y,
            };
        }
        if dy.abs() < dx.abs() {
            return GestureOutcome::Cancelled;
        }

        let velocity = velocity(dy, duration_ms);
        let is_fast = velocity.unsigned_abs() >= u32::from(self.config.velocity_px_per_s);
        let is_far = dy.unsigned_abs() >= u32::from(self.config.threshold_px);
        if !is_fast && !is_far {
            return GestureOutcome::Cancelled;
        }

        GestureOutcome::Swipe(Swipe {
            origin: self.origin,
            direction: if dy < 0 {
                SwipeDirection::Up
            } else {
                SwipeDirection::Down
            },
            distance_px: dy.unsigned_abs().min(u32::from(u16::MAX)) as u16,
            velocity_px_per_s: velocity,
        })
    }
}

fn edge_origin(y: u16, viewport_height: u16, margin: u16) -> SwipeOrigin {
    if y < margin {
        SwipeOrigin::TopEdge
    } else if y >= viewport_height.saturating_sub(margin) {
        SwipeOrigin::BottomEdge
    } else {
        SwipeOrigin::Content
    }
}

fn velocity(dy: i32, duration_ms: u64) -> i32 {
    let duration = duration_ms.max(1) as i64;
    let value = i64::from(dy).saturating_mul(1_000) / duration;
    value.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}
