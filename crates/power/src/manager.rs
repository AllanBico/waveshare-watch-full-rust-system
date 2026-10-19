//! The power manager: applies rail plans, audits them, and tracks the budget.

use common::{Mode, RenderPolicy, WakeReason};
use hal::Power;

use super::budget::{Budget, BudgetStatus};
use super::error::PowerError;
use super::plan::{PowerState, RailPlan, ENABLE_ORDER};
use super::sleep::SleepPlan;

/// Default deep-sleep timer interval.
const DEFAULT_SLEEP_TIMER_MS: u32 = 10_000;

/// Owns the PMIC handle, the runtime mode, the render policy, and the wake
/// budget. Every wake and sleep transition goes through here.
pub struct PowerManager<P> {
    pmic: P,
    mode: Mode,
    state: PowerState,
    policy: RenderPolicy,
    last_wake: Option<WakeReason>,
    budget: Budget,
    plan: RailPlan,
    sleep_timer_ms: u32,
}

impl<P: Power> PowerManager<P> {
    /// Creates a manager for `mode`. Does not touch hardware until a
    /// transition is requested.
    pub const fn new(pmic: P, mode: Mode) -> Self {
        Self {
            pmic,
            mode,
            state: PowerState::for_mode(mode),
            policy: RenderPolicy::for_mode(mode),
            last_wake: None,
            budget: Budget::idle(),
            plan: RailPlan::none(),
            sleep_timer_ms: DEFAULT_SLEEP_TIMER_MS,
        }
    }

    /// Reads the actual rail state from the PMIC into the manager's plan.
    ///
    /// Must be called once after construction: the PMIC powers up with rails
    /// already enabled, and rails persist across deep sleep. Assuming an
    /// initial "all off" state would make the audit fail on the first
    /// transition.
    pub fn sync(&mut self) -> Result<(), PowerError<P::Error>> {
        let mut mask = 0u16;
        for rail in ENABLE_ORDER {
            if self.pmic.is_rail_enabled(rail).map_err(PowerError::Pmic)? {
                mask |= 1 << rail.index();
            }
        }
        self.plan = RailPlan::from_mask(mask);
        Ok(())
    }

    /// Switches runtime mode, applying the new state's rail plan and render
    /// policy. Mode is chosen from VBUS detection, never a build variant.
    pub fn set_mode(&mut self, mode: Mode) -> Result<(), PowerError<P::Error>> {
        self.mode = mode;
        self.state = PowerState::for_mode(mode);
        self.policy = RenderPolicy::for_mode(mode);
        self.apply_plan(self.state.rail_plan())
    }

    /// Marks the device awake for `reason`, applies the awake plan, and starts
    /// the awake budget.
    pub fn request_awake(
        &mut self,
        reason: WakeReason,
        budget_ms: u32,
        now_ms: u64,
    ) -> Result<(), PowerError<P::Error>> {
        self.last_wake = Some(reason);
        self.state = PowerState::for_mode(self.mode);
        self.policy = RenderPolicy::for_mode(self.mode);
        self.budget.start(now_ms, budget_ms);
        self.apply_plan(self.state.rail_plan())
    }

    /// Applies the deep-sleep plan and stops the budget.
    pub fn enter_sleep(&mut self) -> Result<(), PowerError<P::Error>> {
        self.state = PowerState::DeepSleep;
        self.policy = RenderPolicy::WATCH;
        self.budget.stop();
        self.apply_plan(PowerState::DeepSleep.rail_plan())
    }

    /// Applies the deep-sleep rail plan and returns the SoC sleep policy for
    /// the firmware to execute.
    pub fn plan_sleep(&mut self) -> Result<SleepPlan, PowerError<P::Error>> {
        self.plan_sleep_in(self.sleep_timer_ms)
    }

    /// Sleeps with an explicit timer, used when an alarm is earlier than the
    /// housekeeping interval.
    pub fn plan_sleep_in(&mut self, timer_ms: u32) -> Result<SleepPlan, PowerError<P::Error>> {
        self.enter_sleep()?;
        Ok(SleepPlan {
            timer_ms,
            wake_on_button: true,
            rails: self.plan,
        })
    }

    /// Evaluates the awake budget at `now_ms`.
    pub const fn check_budget(&self, now_ms: u64) -> BudgetStatus {
        self.budget.status(now_ms)
    }

    /// Whether USB (VBUS) is present.
    pub fn is_vbus_present(&mut self) -> Result<bool, PowerError<P::Error>> {
        self.pmic.is_vbus_present().map_err(PowerError::Pmic)
    }

    /// Applies a plan: power-down in reverse order, power-up forward, then audit.
    fn apply_plan(&mut self, desired: RailPlan) -> Result<(), PowerError<P::Error>> {
        let previous = self.plan;
        for rail in ENABLE_ORDER.iter().rev() {
            if previous.wants_on(*rail) && !desired.wants_on(*rail) {
                self.pmic
                    .set_rail_enabled(*rail, false)
                    .map_err(PowerError::Pmic)?;
            }
        }
        for rail in ENABLE_ORDER.iter() {
            if !previous.wants_on(*rail) && desired.wants_on(*rail) {
                self.pmic
                    .set_rail_enabled(*rail, true)
                    .map_err(PowerError::Pmic)?;
            }
        }
        self.audit(desired)?;
        self.plan = desired;
        Ok(())
    }

    /// Verifies every rail matches the plan, reading the hardware back.
    fn audit(&mut self, desired: RailPlan) -> Result<(), PowerError<P::Error>> {
        for rail in ENABLE_ORDER.iter() {
            let actual = self.pmic.is_rail_enabled(*rail).map_err(PowerError::Pmic)?;
            if actual != desired.wants_on(*rail) {
                return Err(PowerError::AuditFailed);
            }
        }
        Ok(())
    }

    /// The current logical power state.
    pub const fn state(&self) -> PowerState {
        self.state
    }

    /// The current runtime mode.
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The current render policy.
    pub const fn policy(&self) -> RenderPolicy {
        self.policy
    }

    /// The reason recorded for the most recent wake.
    pub const fn last_wake(&self) -> Option<WakeReason> {
        self.last_wake
    }

    /// The configured RTC-timer sleep interval.
    pub const fn sleep_timer_ms(&self) -> u32 {
        self.sleep_timer_ms
    }

    /// Sets the RTC-timer sleep interval.
    pub fn set_sleep_timer_ms(&mut self, millis: u32) {
        self.sleep_timer_ms = millis;
    }

    /// The rails the manager believes are enabled.
    pub const fn plan(&self) -> RailPlan {
        self.plan
    }

    /// Mutable access to the PMIC (for subsystem-specific setup).
    pub fn pmic_mut(&mut self) -> &mut P {
        &mut self.pmic
    }
}
