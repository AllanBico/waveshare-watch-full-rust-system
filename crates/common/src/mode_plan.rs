//! Per-mode boot capabilities.
//!
//! `firmware` runs one boot chain. This plan names the only steps that differ
//! between watch and desk, so there are no duplicate `watch_init`/`desk_init`
//! bodies.

use crate::Mode;

/// The boot steps that differ between modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModePlan {
    /// Desk initializes PSRAM; watch never touches it.
    pub initialize_psram: bool,
    /// Desk initializes touch; watch leaves it alone.
    pub initialize_touch: bool,
    /// Desk logs over serial; watch stays quiet.
    pub logging: bool,
}

impl ModePlan {
    /// Builds the capability plan for `mode`.
    pub const fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Watch => Self {
                initialize_psram: false,
                initialize_touch: false,
                logging: false,
            },
            Mode::Desk => Self {
                initialize_psram: true,
                initialize_touch: true,
                logging: true,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_boot_stays_minimal_and_quiet() {
        let plan = ModePlan::for_mode(Mode::Watch);
        assert!(!plan.initialize_psram);
        assert!(!plan.initialize_touch);
        assert!(!plan.logging);
    }

    #[test]
    fn desk_boot_initializes_its_extras() {
        let plan = ModePlan::for_mode(Mode::Desk);
        assert!(plan.initialize_psram);
        assert!(plan.initialize_touch);
        assert!(plan.logging);
    }
}
