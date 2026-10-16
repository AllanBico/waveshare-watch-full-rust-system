//! `hal::Power` adapter for the AXP2101.

use common::Millivolts;
use embedded_hal::i2c::I2c;
use hal::{Power, Rail};

use super::{Axp2101, AxpError};

impl<I: I2c> Power for Axp2101<I> {
    type Error = AxpError<I::Error>;

    fn set_rail_enabled(&mut self, rail: Rail, enabled: bool) -> Result<(), Self::Error> {
        Axp2101::enable_rail(self, rail, enabled)
    }

    fn is_rail_enabled(&mut self, rail: Rail) -> Result<bool, Self::Error> {
        Axp2101::is_rail_enabled(self, rail)
    }

    fn set_rail_voltage(&mut self, rail: Rail, millivolts: Millivolts) -> Result<(), Self::Error> {
        Axp2101::set_rail_voltage(self, rail, millivolts)
    }

    fn read_battery_millivolts(&mut self) -> Result<Millivolts, Self::Error> {
        Axp2101::read_battery_millivolts(self)
    }

    fn is_vbus_present(&mut self) -> Result<bool, Self::Error> {
        Ok(Axp2101::read_status(self)?.vbus_good)
    }

    fn power_off(&mut self) -> Result<(), Self::Error> {
        Axp2101::power_off(self)
    }
}
