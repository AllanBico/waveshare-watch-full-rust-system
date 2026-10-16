//! Power/rail abstraction.
//!
//! The rail set is the board's controllable outputs. Concrete PMIC drivers
//! implement [`Power`]; `power` drives it and owns all policy.

use common::Millivolts;

/// A controllable power output.
///
/// Only outputs wired on this board. The discriminant is a dense index used
/// for bitmask rail plans.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[repr(u8)]
pub enum Rail {
    /// DCDC1 (system 3.3 V).
    Dcdc1 = 0,
    /// DCDC2.
    Dcdc2 = 1,
    /// DCDC3.
    Dcdc3 = 2,
    /// DCDC4.
    Dcdc4 = 3,
    /// ALDO1.
    Aldo1 = 4,
    /// ALDO2.
    Aldo2 = 5,
    /// ALDO3 (peripheral 3.3 V: motor and VL peripherals).
    Aldo3 = 6,
    /// ALDO4.
    Aldo4 = 7,
    /// BLDO1.
    Bldo1 = 8,
    /// BLDO2.
    Bldo2 = 9,
    /// CPUSLDO.
    Cpusldo = 10,
}

/// Number of controllable rails.
pub const RAIL_COUNT: usize = 11;

impl Rail {
    /// Stable dense index in `0..RAIL_COUNT`.
    pub const fn index(self) -> u8 {
        self as u8
    }
}

/// A power source with switchable, readable rails.
pub trait Power {
    /// Driver-specific error.
    type Error;

    /// Enables or disables a rail.
    fn set_rail_enabled(&mut self, rail: Rail, enabled: bool) -> Result<(), Self::Error>;

    /// Reads back whether a rail is enabled.
    fn is_rail_enabled(&mut self, rail: Rail) -> Result<bool, Self::Error>;

    /// Sets a rail's output voltage.
    fn set_rail_voltage(&mut self, rail: Rail, millivolts: Millivolts) -> Result<(), Self::Error>;

    /// Reads battery voltage.
    fn read_battery_millivolts(&mut self) -> Result<Millivolts, Self::Error>;

    /// Whether VBUS (USB) is present.
    fn is_vbus_present(&mut self) -> Result<bool, Self::Error>;

    /// Powers the PMIC off.
    fn power_off(&mut self) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_are_dense_and_stable() {
        assert_eq!(Rail::Dcdc1.index(), 0);
        assert_eq!(Rail::Aldo3.index(), 6);
        assert_eq!(Rail::Cpusldo.index(), (RAIL_COUNT - 1) as u8);
    }
}
