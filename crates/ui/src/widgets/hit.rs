//! Hit targets and deterministic focus traversal.

use common::Region;

use crate::layout::LayoutEntry;

use super::arena::{WidgetArena, WidgetError};
use super::{Widget, WidgetId};

/// Maximum hit targets retained by one screen.
pub const MAX_HIT_TARGETS: usize = 64;

/// One interactive rectangle registered by a widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HitTarget {
    /// Stable widget identifier.
    pub id: WidgetId,
    /// Screen-space rectangle, already clipped to the layout clip.
    pub rect: Region,
    /// Whether focus traversal visits this target.
    pub focusable: bool,
}

impl HitTarget {
    /// An empty placeholder.
    pub const EMPTY: Self = Self {
        id: 0,
        rect: Region::new(0, 0, 0, 0),
        focusable: false,
    };
}

/// A fixed-capacity list of hit targets in paint order.
pub struct HitRegistry {
    targets: [HitTarget; MAX_HIT_TARGETS],
    len: usize,
}

impl HitRegistry {
    /// Creates an empty registry.
    pub const fn new() -> Self {
        Self {
            targets: [HitTarget::EMPTY; MAX_HIT_TARGETS],
            len: 0,
        }
    }

    /// Removes every target.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Registers one target.
    pub fn push(&mut self, target: HitTarget) -> Result<(), WidgetError> {
        let Some(slot) = self.targets.get_mut(self.len) else {
            return Err(WidgetError::HitRegistryFull);
        };
        *slot = target;
        self.len += 1;
        Ok(())
    }

    /// The registered targets in paint order.
    pub fn targets(&self) -> &[HitTarget] {
        &self.targets[..self.len]
    }

    /// The number of targets.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether no targets are registered.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the topmost target containing a point.
    pub fn hit(&self, x: u16, y: u16) -> Option<WidgetId> {
        self.targets()
            .iter()
            .rev()
            .find(|target| target.rect.contains(x, y))
            .map(|target| target.id)
    }

    /// Returns the next focusable target after `current`, wrapping once.
    pub fn focus_next(&self, current: Option<WidgetId>) -> Option<WidgetId> {
        self.focus_step(current, 1)
    }

    /// Returns the previous focusable target before `current`, wrapping once.
    pub fn focus_prev(&self, current: Option<WidgetId>) -> Option<WidgetId> {
        self.focus_step(current, -1)
    }

    fn focus_step(&self, current: Option<WidgetId>, step: isize) -> Option<WidgetId> {
        let targets = self.targets();
        if targets.is_empty() {
            return None;
        }
        let start = current
            .and_then(|id| targets.iter().position(|target| target.id == id))
            .map(|index| index as isize + step)
            .unwrap_or(if step > 0 {
                0
            } else {
                targets.len() as isize - 1
            });
        for offset in 0..targets.len() {
            let index =
                (start + step * offset as isize).rem_euclid(targets.len() as isize) as usize;
            let target = targets[index];
            if target.focusable {
                return Some(target.id);
            }
        }
        None
    }
}

impl Default for HitRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registers every widget that has a nonzero layout entry.
pub fn collect_hits(
    widgets: &WidgetArena<'_>,
    entries: &[LayoutEntry],
    registry: &mut HitRegistry,
) -> Result<(), WidgetError> {
    registry.clear();
    for entry in entries {
        let Some(widget) = widgets.find(entry.id) else {
            continue;
        };
        let rect = entry.rect.visible(entry.clip);
        if rect.is_empty() {
            continue;
        }
        registry.push(HitTarget {
            id: widget.id,
            rect,
            focusable: is_focusable(widget),
        })?;
    }
    Ok(())
}

fn is_focusable(widget: &Widget<'_>) -> bool {
    widget.kind.is_focusable() && !widget.state.is_disabled()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{ButtonProps, WidgetKind, WidgetState};

    fn button(id: u16, state: WidgetState) -> Widget<'static> {
        Widget::new(
            id,
            WidgetKind::Button(ButtonProps {
                label: "ok",
                icon: None,
            }),
        )
        .with_state(state)
    }

    fn targets(items: &[(u16, bool)]) -> HitRegistry {
        let mut registry = HitRegistry::new();
        for (index, &(id, focusable)) in items.iter().enumerate() {
            registry
                .push(HitTarget {
                    id,
                    rect: Region::new(0, index as u16 * 10, 100, 10),
                    focusable,
                })
                .unwrap();
        }
        registry
    }

    #[test]
    fn hit_testing_prefers_the_topmost_target() {
        let registry = targets(&[(1, true), (2, true)]);
        assert_eq!(registry.hit(5, 5), Some(1));
        assert_eq!(registry.hit(5, 15), Some(2));
        assert_eq!(registry.hit(500, 500), None);
    }

    #[test]
    fn focus_traversal_skips_non_focusable_targets_and_wraps() {
        let registry = targets(&[(1, true), (2, false), (3, true), (4, true)]);
        assert_eq!(registry.focus_next(None), Some(1));
        assert_eq!(registry.focus_next(Some(1)), Some(3));
        assert_eq!(registry.focus_next(Some(4)), Some(1));
        assert_eq!(registry.focus_prev(Some(1)), Some(4));
        assert_eq!(registry.focus_prev(None), Some(4));
    }

    #[test]
    fn disabled_widgets_are_not_focusable() {
        let mut widgets = WidgetArena::new();
        widgets.push(button(1, WidgetState::NORMAL)).unwrap();
        widgets
            .push(button(2, WidgetState::NORMAL.with(WidgetState::DISABLED)))
            .unwrap();
        let entries = [entry(1), entry(2)];
        let mut registry = HitRegistry::new();
        collect_hits(&widgets, &entries, &mut registry).unwrap();
        assert_eq!(registry.targets().len(), 2);
        assert_eq!(registry.focus_next(None), Some(1));
        assert_eq!(registry.focus_next(Some(1)), Some(1));
    }

    fn entry(id: u16) -> LayoutEntry {
        LayoutEntry {
            id,
            rect: crate::layout::LayoutRect::new(i32::from(id) * 10, 0, 10, 10),
            clip: Region::new(0, 0, 100, 100),
        }
    }
}
