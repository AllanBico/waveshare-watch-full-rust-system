//! Dirty-region collection, merging, and capping.

use common::Region;

/// Maximum dirty rectangles retained for one frame.
pub const MAX_DAMAGE_RECTS: usize = 8;

const EMPTY_REGION: Region = Region::new(0, 0, 0, 0);

/// Dirty regions for one frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Damage {
    regions: [Region; MAX_DAMAGE_RECTS],
    len: usize,
    full: bool,
}

impl Damage {
    /// Creates empty damage.
    pub const fn new() -> Self {
        Self {
            regions: [EMPTY_REGION; MAX_DAMAGE_RECTS],
            len: 0,
            full: false,
        }
    }

    /// Creates a full-screen redraw.
    pub fn full(viewport: Region) -> Self {
        let mut damage = Self::new();
        damage.full = true;
        if !viewport.is_empty() {
            damage.regions[0] = viewport;
            damage.len = 1;
        }
        damage
    }

    /// Adds a dirty region, merging it with any overlapping or touching ones.
    pub fn add(&mut self, region: Region) {
        if self.full || region.is_empty() {
            return;
        }
        let mut merged = region;
        let mut index = 0;
        while index < self.len {
            if touches(self.regions[index], merged) {
                merged = self.regions[index].union(merged);
                self.remove(index);
            } else {
                index += 1;
            }
        }
        self.insert(merged);
        self.coalesce();
    }

    /// The dirty regions.
    pub fn regions(&self) -> &[Region] {
        &self.regions[..self.len]
    }

    /// Whether this frame is a deliberate full redraw.
    pub const fn is_full(&self) -> bool {
        self.full
    }

    /// Whether there is nothing to repaint.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Total dirty pixels across all regions.
    pub fn total_pixels(&self) -> u32 {
        self.regions().iter().fold(0u32, |total, region| {
            total.saturating_add(region.pixel_count())
        })
    }

    fn insert(&mut self, region: Region) {
        if self.len < MAX_DAMAGE_RECTS {
            self.regions[self.len] = region;
            self.len += 1;
            return;
        }
        let mut best = 0;
        let mut best_area = u32::MAX;
        for (index, existing) in self.regions.iter().enumerate() {
            let area = existing.union(region).pixel_count();
            if area < best_area {
                best_area = area;
                best = index;
            }
        }
        self.regions[best] = self.regions[best].union(region);
    }

    fn remove(&mut self, index: usize) {
        self.len -= 1;
        self.regions[index] = self.regions[self.len];
    }

    fn coalesce(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            'search: for first in 0..self.len {
                for second in (first + 1)..self.len {
                    if touches(self.regions[first], self.regions[second]) {
                        self.regions[first] = self.regions[first].union(self.regions[second]);
                        self.remove(second);
                        changed = true;
                        break 'search;
                    }
                }
            }
        }
    }
}

impl Default for Damage {
    fn default() -> Self {
        Self::new()
    }
}

fn touches(first: Region, second: Region) -> bool {
    first.x <= second.right()
        && second.x <= first.right()
        && first.y <= second.bottom()
        && second.y <= first.bottom()
}
