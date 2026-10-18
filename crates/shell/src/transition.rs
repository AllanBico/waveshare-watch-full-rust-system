//! Overlay enter and exit transitions.

use super::LayerKind;

/// One in-flight overlay transition.
///
/// Watch mode reports no transition. Desk mode exposes progress so the
/// renderer can animate without owning navigation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerTransition {
    kind: LayerKind,
    is_opening: bool,
    start_ms: u64,
    duration_ms: u16,
}

impl LayerTransition {
    /// Creates a transition starting at `start_ms`.
    pub const fn new(kind: LayerKind, is_opening: bool, start_ms: u64, duration_ms: u16) -> Self {
        Self {
            kind,
            is_opening,
            start_ms,
            duration_ms,
        }
    }

    /// The layer being animated.
    pub const fn kind(self) -> LayerKind {
        self.kind
    }

    /// Whether the layer is opening rather than closing.
    pub const fn is_opening(self) -> bool {
        self.is_opening
    }

    /// The transition duration.
    pub const fn duration_ms(self) -> u16 {
        self.duration_ms
    }

    /// Completion percentage in `0..=100`.
    pub fn progress(self, now_ms: u64) -> u8 {
        if self.duration_ms == 0 {
            return 100;
        }
        let duration = u64::from(self.duration_ms);
        let elapsed = now_ms.saturating_sub(self.start_ms).min(duration);
        ((elapsed * 100) / duration) as u8
    }

    /// Whether the transition has reached its end.
    pub fn is_finished(self, now_ms: u64) -> bool {
        self.progress(now_ms) >= 100
    }
}
