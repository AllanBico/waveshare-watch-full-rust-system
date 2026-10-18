//! Layout result entries and the fixed-capacity output writer.

use common::Region;

use super::node::NodeId;
use super::rect::LayoutRect;
use super::LayoutError;

/// One laid-out node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutEntry {
    /// Stable node identifier.
    pub id: NodeId,
    /// Unclipped rectangle in layout space.
    pub rect: LayoutRect,
    /// Screen region this node may paint inside.
    pub clip: Region,
}

/// A borrowed, fixed-capacity entry writer.
pub(crate) struct LayoutOutput<'a> {
    entries: &'a mut [LayoutEntry],
    count: usize,
}

impl<'a> LayoutOutput<'a> {
    /// Creates an empty writer over caller storage.
    pub(crate) const fn new(entries: &'a mut [LayoutEntry]) -> Self {
        Self { entries, count: 0 }
    }

    /// Appends one entry, failing when the output slice is full.
    pub(crate) fn push(
        &mut self,
        id: NodeId,
        rect: LayoutRect,
        clip: Region,
    ) -> Result<(), LayoutError> {
        let Some(slot) = self.entries.get_mut(self.count) else {
            return Err(LayoutError::OutputFull);
        };
        *slot = LayoutEntry { id, rect, clip };
        self.count += 1;
        Ok(())
    }

    /// The number of entries written.
    pub(crate) const fn len(&self) -> usize {
        self.count
    }
}
