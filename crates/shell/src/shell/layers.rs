//! Overlay opening, closing, and animated transitions.

use ui::Theme;

use crate::{Layer, LayerKind, LayerTransition, ShellError};

use super::Shell;
use crate::AppRegistry;

impl<R: AppRegistry> Shell<R> {
    pub(super) fn open_layer(&mut self, kind: LayerKind, id: u16) {
        self.layers.open(Layer::new(kind, id));
        self.start_transition(kind, true);
    }

    pub(super) fn close_layer(&mut self, kind: LayerKind) {
        if !self.layers.is_open(kind) {
            return;
        }
        if self.transition_duration() == 0 {
            self.layers.close(kind);
            self.transition = None;
        } else {
            self.start_transition(kind, false);
        }
    }

    pub(super) fn expire_toast(&mut self, now_ms: u64) {
        let Some(deadline) = self.toast_deadline else {
            return;
        };
        if now_ms >= deadline {
            self.layers.close(LayerKind::Toast);
            self.toast_deadline = None;
        }
    }

    pub(super) fn show_toast(&mut self, id: u16, duration_ms: u32) -> Result<(), ShellError> {
        self.layers.open(Layer::new(LayerKind::Toast, id));
        self.toast_deadline = Some(self.uptime_ms.saturating_add(u64::from(duration_ms)));
        Ok(())
    }

    pub(super) fn advance_transition(&mut self, now_ms: u64) {
        let Some(transition) = self.transition else {
            return;
        };
        if transition.is_finished(now_ms) {
            if !transition.is_opening() {
                self.layers.close(transition.kind());
            }
            self.transition = None;
        }
    }

    fn start_transition(&mut self, kind: LayerKind, is_opening: bool) {
        let duration = self.transition_duration();
        if duration == 0 {
            self.transition = None;
            if !is_opening {
                self.layers.close(kind);
            }
        } else {
            self.transition = Some(LayerTransition::new(
                kind,
                is_opening,
                self.uptime_ms,
                duration,
            ));
        }
    }

    fn transition_duration(&self) -> u16 {
        if self.policy.is_animated {
            Theme::for_pixel_mode(self.policy.pixel).motion.slow_ms
        } else {
            0
        }
    }
}
