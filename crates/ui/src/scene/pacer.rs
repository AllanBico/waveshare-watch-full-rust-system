//! Desk frame pacing.

/// Minimum interval between animated desk frames, in milliseconds.
pub const DESK_FRAME_INTERVAL_MS: u64 = 16;

/// Caps animated frames at 60 fps.
#[derive(Debug, Clone, Copy, Default)]
pub struct FramePacer {
    last_ms: Option<u64>,
}

impl FramePacer {
    /// Creates a pacer with no previous frame.
    pub const fn new() -> Self {
        Self { last_ms: None }
    }

    /// Clears the last-frame timestamp.
    pub fn reset(&mut self) {
        self.last_ms = None;
    }

    /// Whether a frame may render at `now_ms`.
    ///
    /// Non-animated modes always pass. Animated modes are capped to 60 fps.
    pub fn allow(&mut self, now_ms: u64, is_animated: bool) -> bool {
        if !is_animated {
            self.last_ms = Some(now_ms);
            return true;
        }
        match self.last_ms {
            Some(last) if now_ms.saturating_sub(last) < DESK_FRAME_INTERVAL_MS => false,
            _ => {
                self.last_ms = Some(now_ms);
                true
            }
        }
    }
}
