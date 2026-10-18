//! Deterministic fill distribution and spacing totals.

/// Deterministic integer distribution of remaining fill space.
pub(crate) struct FillCursor {
    remaining: u16,
    total_weight: u32,
    used_weight: u32,
    previous: u32,
}

impl FillCursor {
    /// Creates a cursor for `remaining` pixels and `total_weight` weight.
    pub(crate) const fn new(remaining: u16, total_weight: u32) -> Self {
        Self {
            remaining,
            total_weight,
            used_weight: 0,
            previous: 0,
        }
    }

    /// Takes the next weighted share.
    pub(crate) fn take(&mut self, weight: u16) -> u16 {
        if self.total_weight == 0 {
            return 0;
        }
        self.used_weight = self.used_weight.saturating_add(u32::from(weight.max(1)));
        let scaled = u64::from(self.remaining) * u64::from(self.used_weight);
        let boundary =
            (scaled / u64::from(self.total_weight)).min(u64::from(self.remaining)) as u32;
        let share = boundary.saturating_sub(self.previous);
        self.previous = boundary;
        share.min(u32::from(u16::MAX)) as u16
    }
}

/// The total spacing between `count` items.
pub(crate) fn gap_total(count: usize, spacing: u16) -> u32 {
    let gaps = u32::try_from(count.saturating_sub(1)).unwrap_or(u32::MAX);
    gaps.saturating_mul(u32::from(spacing))
}
