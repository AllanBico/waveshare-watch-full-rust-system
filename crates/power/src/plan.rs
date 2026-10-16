//! Power states and their rail plans.
//!
//! A `RailPlan` is the desired enable state of every controllable rail. It is
//! the single source of truth the manager applies and audits.

use common::Mode;
use hal::Rail;

/// A logical power state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum PowerState {
    /// Deep sleep: SoC off, only wake-critical rails on.
    DeepSleep,
    /// Active watch: on wrist, monochrome UI.
    ActiveWatch,
    /// Always-on display: low-brightness clock.
    Aod,
    /// Desk: USB powered, full peripherals.
    Desk,
}

impl PowerState {
    /// The default state for a runtime mode.
    pub const fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Watch => Self::ActiveWatch,
            Mode::Desk => Self::Desk,
        }
    }

    /// The rail plan for this state.
    ///
    /// Provisional pending the hardware rail audit (which rail feeds the IMU
    /// and the display). Frozen once measured.
    pub const fn rail_plan(self) -> RailPlan {
        match self {
            Self::DeepSleep => SLEEP_PLAN,
            Self::ActiveWatch | Self::Aod => ACTIVE_PLAN,
            Self::Desk => DESK_PLAN,
        }
    }
}

/// The desired enable state of every controllable rail (bit `i` = rail `i` on).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RailPlan(u16);

impl RailPlan {
    /// No rails enabled.
    pub const fn none() -> Self {
        Self(0)
    }

    /// A copy of this plan with `rail` enabled.
    pub const fn with(mut self, rail: Rail) -> Self {
        self.0 |= 1 << rail.index();
        self
    }

    /// Builds a plan from a raw bitmask.
    pub const fn from_mask(mask: u16) -> Self {
        Self(mask)
    }

    /// Whether `rail` should be enabled under this plan.
    pub const fn wants_on(self, rail: Rail) -> bool {
        self.0 & (1 << rail.index()) != 0
    }

    /// The raw bitmask.
    pub const fn mask(self) -> u16 {
        self.0
    }
}

/// Rails kept alive in deep sleep: system 3.3 V, sensors, and RTC/display
/// enable support. Everything else is cut. Provisional pending the rail audit.
const SLEEP_PLAN: RailPlan = RailPlan::none()
    .with(Rail::Dcdc1)
    .with(Rail::Aldo1)
    .with(Rail::Aldo2)
    .with(Rail::Aldo4);

/// Rails enabled while awake on battery.
const ACTIVE_PLAN: RailPlan = RailPlan::none()
    .with(Rail::Dcdc1)
    .with(Rail::Dcdc3)
    .with(Rail::Dcdc4)
    .with(Rail::Aldo1)
    .with(Rail::Aldo2)
    .with(Rail::Aldo3)
    .with(Rail::Aldo4)
    .with(Rail::Bldo1);

/// All rails enabled in desk mode.
const DESK_PLAN: RailPlan = RailPlan::none()
    .with(Rail::Dcdc1)
    .with(Rail::Dcdc2)
    .with(Rail::Dcdc3)
    .with(Rail::Dcdc4)
    .with(Rail::Aldo1)
    .with(Rail::Aldo2)
    .with(Rail::Aldo3)
    .with(Rail::Aldo4)
    .with(Rail::Bldo1)
    .with(Rail::Bldo2)
    .with(Rail::Cpusldo);

/// Power-up order; power-down is the reverse.
pub(super) const ENABLE_ORDER: [Rail; 11] = [
    Rail::Dcdc1,
    Rail::Dcdc2,
    Rail::Dcdc3,
    Rail::Dcdc4,
    Rail::Aldo1,
    Rail::Aldo2,
    Rail::Aldo3,
    Rail::Aldo4,
    Rail::Bldo1,
    Rail::Bldo2,
    Rail::Cpusldo,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_cuts_peripheral_rail() {
        let plan = PowerState::DeepSleep.rail_plan();
        assert!(plan.wants_on(Rail::Dcdc1));
        assert!(!plan.wants_on(Rail::Aldo3));
    }

    #[test]
    fn desk_enables_every_rail() {
        let plan = PowerState::Desk.rail_plan();
        for rail in ENABLE_ORDER {
            assert!(plan.wants_on(rail));
        }
    }

    #[test]
    fn with_sets_the_bit() {
        let plan = RailPlan::none().with(Rail::Aldo3);
        assert_eq!(plan.mask(), 1 << Rail::Aldo3.index());
        assert!(plan.wants_on(Rail::Aldo3));
        assert!(!plan.wants_on(Rail::Dcdc1));
    }
}
