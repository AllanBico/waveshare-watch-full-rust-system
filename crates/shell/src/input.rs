//! Touch-report adapter for the edge-gesture state machine.

use common::TouchReport;

use super::{EdgeGesture, GestureConfig, GestureOutcome};

/// Feeds touch reports into one edge recognizer.
pub struct InputRouter {
    gesture: EdgeGesture,
    active_id: Option<u8>,
}

impl InputRouter {
    /// Creates an idle router.
    pub const fn new(config: GestureConfig) -> Self {
        Self {
            gesture: EdgeGesture::new(config),
            active_id: None,
        }
    }

    /// The recognizer state.
    pub const fn gesture(&self) -> &EdgeGesture {
        &self.gesture
    }

    /// Handles one controller report.
    pub fn handle(
        &mut self,
        report: &TouchReport,
        now_ms: u64,
        viewport_height: u16,
    ) -> GestureOutcome {
        let active = match self.active_id {
            Some(id) => report.points().iter().find(|point| point.id == id).copied(),
            None => report.points().first().copied(),
        };
        match (self.active_id, active) {
            (None, Some(point)) => {
                self.active_id = Some(point.id);
                self.gesture.begin(point, now_ms, viewport_height);
                GestureOutcome::None
            }
            (Some(_), Some(point)) => {
                self.gesture.update(point, now_ms);
                GestureOutcome::None
            }
            (Some(_), None) => {
                self.active_id = None;
                self.gesture.end(now_ms)
            }
            (None, None) => GestureOutcome::None,
        }
    }

    /// Drops the current touch, for example when a dialog opens.
    pub fn cancel(&mut self) {
        self.active_id = None;
        self.gesture.cancel();
    }
}
