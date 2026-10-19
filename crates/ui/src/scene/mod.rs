//! Scene arena, frame diffing, damage, and frame pacing.
//!
//! The scene stores one flat, fixed-capacity list of laid-out nodes. Diffing a
//! new list against the previous frame produces dirty screen regions without
//! allocating.

mod arena;
mod damage;
mod pacer;
#[cfg(test)]
mod tests;

use common::Region;
use core::fmt;

use crate::layout::LayoutRect;

pub use arena::{SceneArena, SceneNode, WidgetId, SCENE_CAPACITY};
pub use damage::{Damage, MAX_DAMAGE_RECTS};
pub use pacer::{FramePacer, DESK_FRAME_INTERVAL_MS};

/// The only events allowed to force a full-screen redraw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedrawReason {
    /// First frame after boot.
    Boot,
    /// A watch/desk transition.
    ModeSwitch,
    /// A compiled-in theme change.
    ThemeSwitch,
    /// An overlay was dismissed and the base screen must be restored.
    Overlay,
}

/// A scene failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneError {
    /// The arena cannot hold any more nodes.
    ArenaFull,
}

impl fmt::Display for SceneError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("scene arena full")
    }
}

/// Owns the previous-frame snapshot and pending full-redraw state.
pub struct Scene {
    previous: SceneArena,
    has_previous: bool,
    pending_full: Option<RedrawReason>,
}

impl Scene {
    /// Creates an empty scene. The first diff is a full redraw.
    pub const fn new() -> Self {
        Self {
            previous: SceneArena::new(),
            has_previous: false,
            pending_full: None,
        }
    }

    /// Whether at least one frame has been committed.
    pub const fn has_frame(&self) -> bool {
        self.has_previous
    }

    /// Requests a full redraw on the next diff.
    pub fn request_full(&mut self, reason: RedrawReason) {
        self.pending_full = Some(reason);
    }

    /// Diffs `current` against the previous frame and commits it.
    pub fn diff(&mut self, current: &SceneArena, viewport: Region) -> Damage {
        let mut damage = Damage::new();
        let pending = self.pending_full.take().is_some();
        let force_full = !self.has_previous || pending;
        if force_full {
            damage = Damage::full(viewport);
        } else {
            collect_diff(&self.previous, current, viewport, &mut damage);
        }
        self.previous.copy_from(current);
        self.has_previous = true;
        damage
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_diff(
    previous: &SceneArena,
    current: &SceneArena,
    viewport: Region,
    damage: &mut Damage,
) {
    for node in current.nodes() {
        match previous.find(node.id) {
            Some(old) if old.rect != node.rect || old.fingerprint != node.fingerprint => {
                add_rect(damage, old.rect, viewport);
                add_rect(damage, node.rect, viewport);
            }
            Some(_) => {}
            None => add_rect(damage, node.rect, viewport),
        }
    }
    for old in previous.nodes() {
        if current.find(old.id).is_none() {
            add_rect(damage, old.rect, viewport);
        }
    }
}

fn add_rect(damage: &mut Damage, rect: LayoutRect, viewport: Region) {
    damage.add(rect.visible(viewport));
}
