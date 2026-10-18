//! Uniform grid track sizing shared by measure and arrange.

use crate::text::TextCache;

use super::constraint::{Constraints, Size, UNBOUNDED};
use super::fill::gap_total;
use super::measure::measure;
use super::node::Node;
use super::LayoutError;

/// The uniform track geometry of one grid.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GridSpec {
    columns: usize,
    column_spacing: u16,
    available: u16,
}

impl GridSpec {
    /// Validates a grid definition.
    pub(crate) fn new(
        columns: u8,
        column_spacing: u16,
        available: u16,
    ) -> Result<Self, LayoutError> {
        if columns == 0 {
            return Err(LayoutError::InvalidGrid);
        }
        Ok(Self {
            columns: usize::from(columns),
            column_spacing,
            available,
        })
    }

    /// The number of columns.
    pub(crate) const fn columns(self) -> usize {
        self.columns
    }

    /// Gap between adjacent columns.
    pub(crate) const fn column_spacing(self) -> u16 {
        self.column_spacing
    }

    /// The width of one column track.
    pub(crate) fn column_width(
        self,
        children: &[Node<'_>],
        column: usize,
        cache: &mut TextCache,
        depth: u8,
    ) -> Result<u16, LayoutError> {
        if self.available != UNBOUNDED {
            return Ok(bounded_track(
                self.available,
                self.columns,
                self.column_spacing,
                column,
            ));
        }
        let mut width = 0;
        let mut index = column;
        while index < children.len() {
            let size = measure(
                &children[index],
                Constraints::loose(Size::new(UNBOUNDED, UNBOUNDED)),
                cache,
                depth + 1,
            )?;
            width = width.max(size.width);
            index += self.columns;
        }
        Ok(width)
    }

    /// The total width of all tracks and column gaps.
    pub(crate) fn total_width(
        self,
        children: &[Node<'_>],
        cache: &mut TextCache,
        depth: u8,
    ) -> Result<u16, LayoutError> {
        if self.available != UNBOUNDED {
            return Ok(self.available);
        }
        let mut width = gap_total(self.columns, self.column_spacing);
        for column in 0..self.columns {
            width = width.saturating_add(u32::from(
                self.column_width(children, column, cache, depth)?,
            ));
        }
        Ok(width.min(u32::from(u16::MAX)) as u16)
    }
}

fn bounded_track(available: u16, columns: usize, spacing: u16, column: usize) -> u16 {
    let gaps = u32::from(spacing).saturating_mul(columns.saturating_sub(1) as u32);
    let usable = u32::from(available).saturating_sub(gaps);
    let base = usable / columns as u32;
    let extra = usable % columns as u32;
    (base + u32::from(column < extra as usize)).min(u32::from(u16::MAX)) as u16
}
