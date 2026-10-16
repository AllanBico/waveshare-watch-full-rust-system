//! Wake-budget tracking.
//!
//! Time is injected by the caller so the manager stays `no_std` and testable;
//! the firmware owns the actual timer.

/// Whether an active budget has elapsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BudgetStatus {
    /// Still within budget, or no budget is active.
    Awake,
    /// The budget elapsed; the caller must sleep.
    Expired,
}

/// A monotonic awake-time budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budget {
    deadline_ms: u64,
    is_active: bool,
}

impl Budget {
    /// A budget that is not running.
    pub const fn idle() -> Self {
        Self {
            deadline_ms: 0,
            is_active: false,
        }
    }

    /// Starts a budget of `budget_ms` from `now_ms`.
    pub fn start(&mut self, now_ms: u64, budget_ms: u32) {
        self.deadline_ms = now_ms.saturating_add(u64::from(budget_ms));
        self.is_active = true;
    }

    /// Stops the budget (e.g. on sleep).
    pub fn stop(&mut self) {
        self.is_active = false;
    }

    /// Whether a budget is currently running.
    pub const fn is_active(&self) -> bool {
        self.is_active
    }

    /// Evaluates the budget against the current time.
    pub const fn status(&self, now_ms: u64) -> BudgetStatus {
        if self.is_active && now_ms >= self.deadline_ms {
            BudgetStatus::Expired
        } else {
            BudgetStatus::Awake
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_is_always_awake() {
        assert_eq!(Budget::idle().status(1_000_000), BudgetStatus::Awake);
    }

    #[test]
    fn expires_after_deadline() {
        let mut budget = Budget::idle();
        budget.start(1_000, 50);
        assert_eq!(budget.status(1_049), BudgetStatus::Awake);
        assert_eq!(budget.status(1_050), BudgetStatus::Expired);
    }

    #[test]
    fn stop_clears_the_budget() {
        let mut budget = Budget::idle();
        budget.start(0, 10);
        budget.stop();
        assert_eq!(budget.status(999), BudgetStatus::Awake);
    }
}
