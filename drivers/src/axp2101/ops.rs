//! AXP2101 configuration and power-lever operations.

use embedded_hal::i2c::I2c;

use super::regs::{
    ADC_ENABLE, BAT_DETECT, COMMON_CONFIG, COMMON_DISCHARGE, COMMON_POWER_OFF, COMMON_RESET,
    CV_CHG_VOL, DC_DVM_RAMP, DC_FORCE_PWM, DC_ONOFF_DVM, DIE_TEMP_H, IRQ_ENABLE0, IRQ_STATUS0,
    LOW_BAT_SHUTDOWN_MASK, LOW_BAT_WARN, LOW_BAT_WARN_MASK, PWROFF_EN, VOFF, VOLT_HIGH_MASK,
};
use super::{
    AdcChannel, Axp2101, AxpError, ChargeTarget, DcdcMode, DvmRamp, IrqMask, LongPressAction,
    Millivolts, Percent, Rail, RailError,
};

const VOLTAGE_VALUE_MASK: u8 = 0x7F;
const CHARGE_TARGET_MASK: u8 = 0x07;
const BAT_DETECT_MASK: u8 = 0x01;

const VOFF_MIN_MV: u16 = 2600;
const VOFF_MAX_MV: u16 = 3300;
const VOFF_STEP_MV: u16 = 100;
const VOFF_MASK: u8 = 0x07;

const LOW_BAT_WARN_MIN: u8 = 5;
const LOW_BAT_WARN_MAX: u8 = 20;
const LOW_BAT_SHUTDOWN_MAX: u8 = 15;

const PWROFF_LONG_MASK: u8 = 0x03;
const PWROFF_LONG_ENABLE: u8 = 0x02;
const PWROFF_RESTART: u8 = 0x01;

impl<I: I2c> Axp2101<I> {
    /// Sets a rail's output voltage, preserving the DCDC work-mode bit.
    pub fn set_rail_voltage(
        &mut self,
        rail: Rail,
        millivolts: Millivolts,
    ) -> Result<(), AxpError<I::Error>> {
        let code = rail.encode(millivolts)?;
        self.update_u8(rail.voltage_register(), VOLTAGE_VALUE_MASK, code)
    }

    /// Reads back a rail's configured voltage.
    pub fn read_rail_voltage(&mut self, rail: Rail) -> Result<Millivolts, AxpError<I::Error>> {
        let code = self.read_u8(rail.voltage_register())?;
        Ok(rail.decode(code)?)
    }

    /// Enables or disables a rail.
    pub fn enable_rail(&mut self, rail: Rail, on: bool) -> Result<(), AxpError<I::Error>> {
        let mask = 1u8 << rail.enable_bit();
        let value = if on { mask } else { 0 };
        self.update_u8(rail.enable_register(), mask, value)
    }

    /// Reads back whether a rail is enabled.
    pub fn is_rail_enabled(&mut self, rail: Rail) -> Result<bool, AxpError<I::Error>> {
        let mask = 1u8 << rail.enable_bit();
        Ok(self.read_u8(rail.enable_register())? & mask != 0)
    }

    /// Enables or disables one ADC channel.
    pub fn set_adc_channel(
        &mut self,
        channel: AdcChannel,
        on: bool,
    ) -> Result<(), AxpError<I::Error>> {
        let mask = 1u8 << channel.bit();
        let value = if on { mask } else { 0 };
        self.update_u8(ADC_ENABLE, mask, value)
    }

    /// Enables or disables battery detection (required for percent reads).
    pub fn set_battery_detection(&mut self, on: bool) -> Result<(), AxpError<I::Error>> {
        let value = if on { BAT_DETECT_MASK } else { 0 };
        self.update_u8(BAT_DETECT, BAT_DETECT_MASK, value)
    }

    /// Sets the charger constant-voltage target.
    pub fn set_charge_target(&mut self, target: ChargeTarget) -> Result<(), AxpError<I::Error>> {
        self.update_u8(CV_CHG_VOL, CHARGE_TARGET_MASK, target.bits())
    }

    /// Sets the low-battery warning threshold, clamped to 5..=20 percent.
    pub fn set_low_battery_warning(&mut self, percent: Percent) -> Result<(), AxpError<I::Error>> {
        let clamped = percent.value().clamp(LOW_BAT_WARN_MIN, LOW_BAT_WARN_MAX);
        let code = (clamped - LOW_BAT_WARN_MIN) << 4;
        self.update_u8(LOW_BAT_WARN, LOW_BAT_WARN_MASK, code)
    }

    /// Sets the low-battery shutdown threshold, clamped to 0..=15 percent.
    pub fn set_low_battery_shutdown(&mut self, percent: Percent) -> Result<(), AxpError<I::Error>> {
        let clamped = percent.value().min(LOW_BAT_SHUTDOWN_MAX);
        self.update_u8(LOW_BAT_WARN, LOW_BAT_SHUTDOWN_MASK, clamped)
    }

    /// Sets the minimum system voltage before the PMIC shuts down.
    pub fn set_system_shutdown_voltage(
        &mut self,
        millivolts: Millivolts,
    ) -> Result<(), AxpError<I::Error>> {
        let mv = millivolts.value();
        if !(VOFF_MIN_MV..=VOFF_MAX_MV).contains(&mv) {
            return Err(AxpError::Voltage(RailError::OutOfRange));
        }
        if !(mv - VOFF_MIN_MV).is_multiple_of(VOFF_STEP_MV) {
            return Err(AxpError::Voltage(RailError::OffStep));
        }
        let code = ((mv - VOFF_MIN_MV) / VOFF_STEP_MV) as u8;
        self.update_u8(VOFF, VOFF_MASK, code)
    }

    /// Sets a DCDC rail's switching mode.
    pub fn set_dcdc_mode(&mut self, rail: Rail, mode: DcdcMode) -> Result<(), AxpError<I::Error>> {
        let bit = rail
            .dcdc_pwm_bit()
            .ok_or(AxpError::Voltage(RailError::NotApplicable))?;
        let mask = 1u8 << bit;
        let value = if mode == DcdcMode::Pwm { mask } else { 0 };
        self.update_u8(DC_FORCE_PWM, mask, value)
    }

    /// Enables or disables internal output discharge.
    pub fn set_internal_discharge(&mut self, on: bool) -> Result<(), AxpError<I::Error>> {
        let value = if on { COMMON_DISCHARGE } else { 0 };
        self.update_u8(COMMON_CONFIG, COMMON_DISCHARGE, value)
    }

    /// Sets the DVM output slew ramp.
    pub fn set_dvm_ramp(&mut self, ramp: DvmRamp) -> Result<(), AxpError<I::Error>> {
        let value = match ramp {
            DvmRamp::Us15_625 => 0,
            DvmRamp::Us31_250 => DC_DVM_RAMP,
        };
        self.update_u8(DC_ONOFF_DVM, DC_DVM_RAMP, value)
    }

    /// Reads the raw die-temperature ADC code (0.1 mV per LSB).
    pub fn read_die_temperature(&mut self) -> Result<u16, AxpError<I::Error>> {
        let mut buf = [0u8; 2];
        self.read_seq(DIE_TEMP_H, &mut buf)?;
        Ok((((buf[0] & VOLT_HIGH_MASK) as u16) << 8) | buf[1] as u16)
    }

    /// Sets the behavior of a long PWR-button press.
    pub fn set_pwron_long_press(
        &mut self,
        action: LongPressAction,
    ) -> Result<(), AxpError<I::Error>> {
        let action_bit = match action {
            LongPressAction::Restart => PWROFF_RESTART,
            LongPressAction::PowerOff => 0,
        };
        self.update_u8(PWROFF_EN, PWROFF_LONG_MASK, PWROFF_LONG_ENABLE | action_bit)
    }

    /// Enables or disables the interrupts selected in `mask`.
    pub fn set_irq_enabled(&mut self, mask: IrqMask, on: bool) -> Result<(), AxpError<I::Error>> {
        for index in [0u8, 1, 2] {
            let byte = irq_byte(mask, index);
            if byte != 0 {
                let value = if on { byte } else { 0 };
                self.update_u8(IRQ_ENABLE0 + index, byte, value)?;
            }
        }
        Ok(())
    }

    /// Clears the pending interrupts selected in `mask`.
    pub fn clear_irq(&mut self, mask: IrqMask) -> Result<(), AxpError<I::Error>> {
        for index in [0u8, 1, 2] {
            let byte = irq_byte(mask, index);
            if byte != 0 {
                self.write_u8(IRQ_STATUS0 + index, byte)?;
            }
        }
        Ok(())
    }

    /// Reads the pending interrupt flags.
    pub fn pending_irq(&mut self) -> Result<IrqMask, AxpError<I::Error>> {
        let mut buf = [0u8; 3];
        self.read_seq(IRQ_STATUS0, &mut buf)?;
        Ok(IrqMask::from_bytes(buf))
    }

    /// Powers off the PMIC.
    pub fn power_off(&mut self) -> Result<(), AxpError<I::Error>> {
        self.update_u8(COMMON_CONFIG, COMMON_POWER_OFF, COMMON_POWER_OFF)
    }

    /// Resets the PMIC and the SoC.
    pub fn reset_pmic(&mut self) -> Result<(), AxpError<I::Error>> {
        self.update_u8(COMMON_CONFIG, COMMON_RESET, COMMON_RESET)
    }
}

fn irq_byte(mask: IrqMask, index: u8) -> u8 {
    ((mask.bits() >> (8 * u32::from(index))) & 0xFF) as u8
}
