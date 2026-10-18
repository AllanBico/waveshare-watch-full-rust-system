//! Fixed-capacity widget arena.

use super::Widget;

/// Maximum widgets retained by one screen.
pub const WIDGET_CAPACITY: usize = 64;

/// A widget storage or lookup failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetError {
    /// The widget arena is full.
    ArenaFull,
    /// The hit registry is full.
    HitRegistryFull,
}

/// A fixed-capacity list of widgets in paint order.
pub struct WidgetArena<'a> {
    widgets: [Option<Widget<'a>>; WIDGET_CAPACITY],
    len: usize,
}

impl<'a> WidgetArena<'a> {
    /// Creates an empty arena.
    pub const fn new() -> Self {
        Self {
            widgets: [None; WIDGET_CAPACITY],
            len: 0,
        }
    }

    /// The widget capacity.
    pub const fn capacity() -> usize {
        WIDGET_CAPACITY
    }

    /// Removes every widget.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// The number of widgets.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the arena is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends one widget.
    pub fn push(&mut self, widget: Widget<'a>) -> Result<(), WidgetError> {
        let Some(slot) = self.widgets.get_mut(self.len) else {
            return Err(WidgetError::ArenaFull);
        };
        *slot = Some(widget);
        self.len += 1;
        Ok(())
    }

    /// Returns the widget at `index`.
    pub fn get(&self, index: usize) -> Option<&Widget<'a>> {
        self.widgets.get(index)?.as_ref()
    }

    /// Finds a widget by stable id.
    pub fn find(&self, id: super::WidgetId) -> Option<&Widget<'a>> {
        self.iter().find(|widget| widget.id == id)
    }

    /// Iterates widgets in paint order.
    pub fn iter(&self) -> impl Iterator<Item = &Widget<'a>> {
        self.widgets[..self.len].iter().flatten()
    }
}

impl Default for WidgetArena<'_> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widgets::{WidgetKind, WidgetState};

    fn widget(id: u16) -> Widget<'static> {
        Widget::new(
            id,
            WidgetKind::Badge(super::super::BadgeProps { label: "x" }),
        )
    }

    #[test]
    fn push_find_and_clear_round_trip() {
        let mut arena = WidgetArena::new();
        arena.push(widget(1)).unwrap();
        arena.push(widget(2)).unwrap();
        assert_eq!(arena.len(), 2);
        assert_eq!(arena.find(2).map(|widget| widget.id), Some(2));
        assert_eq!(
            arena.get(0).map(|widget| widget.state),
            Some(WidgetState::NORMAL)
        );
        arena.clear();
        assert!(arena.is_empty());
    }
}
