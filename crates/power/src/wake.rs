//! What the device should do for one wake reason.

use common::{Mode, WakeReason};

/// The user-visible action for one wake.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WakeAction {
    /// Present Home and run the awake budget.
    Present,
    /// Do housekeeping and return to sleep without turning on the panel.
    Housekeeping,
}

impl WakeAction {
    /// Classifies a wake for the active mode.
    ///
    /// Watch timer wakes are housekeeping (battery sample, retained log, VBUS
    /// fallback). Motion, button, cold boot, and unknown wakes are visible.
    /// Desk always presents.
    pub const fn for_wake(mode: Mode, wake: Option<WakeReason>) -> Self {
        match mode {
            Mode::Desk => Self::Present,
            Mode::Watch => match wake {
                Some(WakeReason::Timer) => Self::Housekeeping,
                _ => Self::Present,
            },
        }
    }

    /// Whether Home should be shown for this wake.
    pub const fn presents_home(self) -> bool {
        matches!(self, Self::Present)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_timer_wakes_stay_dark() {
        assert_eq!(
            WakeAction::for_wake(Mode::Watch, Some(WakeReason::Timer)),
            WakeAction::Housekeeping
        );
    }

    #[test]
    fn user_wakes_present_home() {
        for wake in [
            None,
            Some(WakeReason::Button),
            Some(WakeReason::Motion),
            Some(WakeReason::Unknown),
        ] {
            assert!(WakeAction::for_wake(Mode::Watch, wake).presents_home());
        }
    }

    #[test]
    fn desk_presents_for_every_wake() {
        for wake in [
            None,
            Some(WakeReason::Timer),
            Some(WakeReason::Button),
            Some(WakeReason::Motion),
            Some(WakeReason::Usb),
            Some(WakeReason::Unknown),
        ] {
            assert!(WakeAction::for_wake(Mode::Desk, wake).presents_home());
        }
    }
}
