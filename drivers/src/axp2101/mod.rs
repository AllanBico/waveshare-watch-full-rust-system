//! AXP2101 power-management driver.
//!
//! Mechanical register access for the AXP2101 PMIC: identity, status,
//! voltages, rails, ADC, charger target, interrupts, and the low-power
//! levers. It performs no power policy; rail sequencing, budgets, and mode
//! decisions belong to the `power` crate.

use embedded_hal::i2c::I2c;

mod adapter;
mod error;
mod irq;
mod ops;
mod rails;
mod regs;
mod status;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub use common::{Millivolts, Percent};
pub use error::{AxpError, RailError};
pub use hal::Rail;
pub use irq::IrqMask;
pub use status::{
    AdcChannel, ChargeState, ChargeTarget, DcdcMode, DvmRamp, LongPressAction, PowerSnapshot,
    Status, WakePowerState,
};

use regs::{CHIP_ID_AXP2101, IC_TYPE, STATUS1, VBAT_H, VBAT_HIGH_MASK, VOLT_HIGH_MASK};

/// An AXP2101 PMIC on a shared I2C bus.
#[derive(Debug)]
pub struct Axp2101<I> {
    i2c: I,
    address: u8,
}

impl<I: I2c> Axp2101<I> {
    /// Probes the AXP2101 at `address`, verifying its chip identity.
    ///
    /// Does not touch any rail. All power policy is applied by the caller.
    pub fn new(i2c: I, address: u8) -> Result<Self, AxpError<I::Error>> {
        let mut device = Self { i2c, address };
        let id = device.read_u8(IC_TYPE)?;
        if id != CHIP_ID_AXP2101 {
            return Err(AxpError::BadChipId(id));
        }
        Ok(device)
    }

    /// Reads the chip identity register.
    pub fn read_chip_id(&mut self) -> Result<u8, AxpError<I::Error>> {
        self.read_u8(IC_TYPE)
    }

    /// Reads and decodes the power status registers.
    pub fn read_status(&mut self) -> Result<Status, AxpError<I::Error>> {
        let mut buf = [0u8; 2];
        self.read_seq(STATUS1, &mut buf)?;
        Ok(Status::from_registers(buf[0], buf[1]))
    }

    /// Reads battery voltage with a single 2-byte transaction (wake path).
    pub fn read_battery_millivolts(&mut self) -> Result<Millivolts, AxpError<I::Error>> {
        let mut buf = [0u8; 2];
        self.read_seq(VBAT_H, &mut buf)?;
        Ok(decode_battery_voltage(buf[0], buf[1]))
    }

    /// Reads the minimal power state needed on the wake path.
    pub fn read_wake_power_state(&mut self) -> Result<WakePowerState, AxpError<I::Error>> {
        let vbus_present = self.read_status()?.vbus_good;
        let battery = self.read_battery_millivolts()?;
        Ok(WakePowerState {
            battery,
            vbus_present,
        })
    }

    /// Reads a consistent voltage and charge sample (desk/UI path).
    ///
    /// All or nothing: any bus error returns `Err` with no partial snapshot.
    pub fn read_power_snapshot(&mut self) -> Result<PowerSnapshot, AxpError<I::Error>> {
        let status = self.read_status()?;
        let mut volts = [0u8; 8];
        self.read_seq(VBAT_H, &mut volts)?;
        let percent = Percent::new(self.read_u8(regs::BAT_PERCENT)?);
        Ok(PowerSnapshot {
            battery: decode_battery_voltage(volts[0], volts[1]),
            vbus: decode_voltage(volts[4], volts[5]),
            system: decode_voltage(volts[6], volts[7]),
            percent,
            status,
        })
    }

    fn read_u8(&mut self, reg: u8) -> Result<u8, AxpError<I::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .map_err(AxpError::Bus)?;
        Ok(buf[0])
    }

    fn write_u8(&mut self, reg: u8, value: u8) -> Result<(), AxpError<I::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(AxpError::Bus)
    }

    fn update_u8(&mut self, reg: u8, mask: u8, value: u8) -> Result<(), AxpError<I::Error>> {
        let current = self.read_u8(reg)?;
        self.write_u8(reg, (current & !mask) | (value & mask))
    }

    fn read_seq(&mut self, reg: u8, buf: &mut [u8]) -> Result<(), AxpError<I::Error>> {
        self.i2c
            .write_read(self.address, &[reg], buf)
            .map_err(AxpError::Bus)
    }
}

fn decode_battery_voltage(high: u8, low: u8) -> Millivolts {
    Millivolts::new((((high & VBAT_HIGH_MASK) as u16) << 8) | low as u16)
}

fn decode_voltage(high: u8, low: u8) -> Millivolts {
    Millivolts::new((((high & VOLT_HIGH_MASK) as u16) << 8) | low as u16)
}
