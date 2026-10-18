//! Fixed-capacity scene nodes and arena.

use crate::layout::{LayoutEntry, LayoutRect, NodeId};

use super::SceneError;

/// Stable identifier carried from layout into the scene.
pub type WidgetId = NodeId;

/// Maximum nodes held by one scene frame.
pub const SCENE_CAPACITY: usize = 128;

/// One laid-out scene node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneNode {
    /// Stable widget identifier.
    pub id: WidgetId,
    /// Unclipped layout rectangle.
    pub rect: LayoutRect,
    /// Hash of the node's content and style.
    pub fingerprint: u64,
}

impl SceneNode {
    /// Creates a scene node.
    pub const fn new(id: WidgetId, rect: LayoutRect, fingerprint: u64) -> Self {
        Self {
            id,
            rect,
            fingerprint,
        }
    }

    const EMPTY: Self = Self::new(0, LayoutRect::new(0, 0, 0, 0), 0);
}

/// A flat, fixed-capacity scene frame.
pub struct SceneArena {
    nodes: [SceneNode; SCENE_CAPACITY],
    len: usize,
}

impl SceneArena {
    /// Creates an empty arena.
    pub const fn new() -> Self {
        Self {
            nodes: [SceneNode::EMPTY; SCENE_CAPACITY],
            len: 0,
        }
    }

    /// The node capacity.
    pub const fn capacity() -> usize {
        SCENE_CAPACITY
    }

    /// Removes every node.
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// The current nodes in paint order.
    pub fn nodes(&self) -> &[SceneNode] {
        &self.nodes[..self.len]
    }

    /// The number of nodes.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the arena has no nodes.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Finds a node by stable id.
    pub fn find(&self, id: WidgetId) -> Option<&SceneNode> {
        self.nodes().iter().find(|node| node.id == id)
    }

    /// Appends one node.
    pub fn push(&mut self, node: SceneNode) -> Result<(), SceneError> {
        let Some(slot) = self.nodes.get_mut(self.len) else {
            return Err(SceneError::ArenaFull);
        };
        *slot = node;
        self.len += 1;
        Ok(())
    }

    /// Rebuilds the arena from layout entries and a fingerprint function.
    pub fn build<F>(&mut self, entries: &[LayoutEntry], fingerprint: F) -> Result<(), SceneError>
    where
        F: FnMut(&LayoutEntry) -> u64,
    {
        if entries.len() > SCENE_CAPACITY {
            return Err(SceneError::ArenaFull);
        }
        self.clear();
        let mut fingerprint = fingerprint;
        for entry in entries {
            self.push(SceneNode::new(entry.id, entry.rect, fingerprint(entry)))?;
        }
        Ok(())
    }

    /// Copies another frame into this arena.
    pub(crate) fn copy_from(&mut self, other: &Self) {
        let len = other.len.min(SCENE_CAPACITY);
        self.nodes[..len].copy_from_slice(&other.nodes[..len]);
        self.len = len;
    }
}

impl Default for SceneArena {
    fn default() -> Self {
        Self::new()
    }
}
