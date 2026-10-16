//! AXP2101 register addresses and bit fields.
//!
//! Register offsets only. The slave address is supplied by the caller. All
//! values are from the AXP2101 datasheet (V1.4) and XPowersLib.

/// Power status register 1.
pub(super) const STATUS1: u8 = 0x00;
/// Chip type / identity register.
pub(super) const IC_TYPE: u8 = 0x03;
/// Common configuration (discharge, power-off, reset).
pub(super) const COMMON_CONFIG: u8 = 0x10;
/// Low battery warning and shutdown thresholds.
pub(super) const LOW_BAT_WARN: u8 = 0x1A;
/// Power-off enable and long-press configuration.
pub(super) const PWROFF_EN: u8 = 0x22;
/// Minimum system voltage before shutdown.
pub(super) const VOFF: u8 = 0x24;
/// ADC channel enable control.
pub(super) const ADC_ENABLE: u8 = 0x30;
/// Battery voltage block start (0x34..0x3B: VBAT, TS, VBUS, VSYS).
pub(super) const VBAT_H: u8 = 0x34;
/// Die temperature ADC block start.
pub(super) const DIE_TEMP_H: u8 = 0x3C;
/// IRQ enable registers (0x40..0x42).
pub(super) const IRQ_ENABLE0: u8 = 0x40;
/// IRQ status registers (0x48..0x4A).
pub(super) const IRQ_STATUS0: u8 = 0x48;
/// Charger constant-voltage target.
pub(super) const CV_CHG_VOL: u8 = 0x64;
/// Battery detection control.
pub(super) const BAT_DETECT: u8 = 0x68;
/// DCDC enable and DVM ramp control.
pub(super) const DC_ONOFF_DVM: u8 = 0x80;
/// DCDC forced-PWM control.
pub(super) const DC_FORCE_PWM: u8 = 0x81;
/// DCDC1 voltage (DCDC1..DCDC4 = 0x82..0x85).
pub(super) const DC_VOL_BASE: u8 = 0x82;
/// LDO enable control 0 (ALDO1..4, BLDO1..2, CPUSLDO).
pub(super) const LDO_ONOFF0: u8 = 0x90;
/// LDO voltage block start (ALDO1..4, BLDO1..2, CPUSLDO = 0x92..0x98).
pub(super) const LDO_VOL_BASE: u8 = 0x92;
/// Battery state of charge.
pub(super) const BAT_PERCENT: u8 = 0xA4;

/// Value returned by [`IC_TYPE`] for the AXP2101.
pub(super) const CHIP_ID_AXP2101: u8 = 0x4A;

/// STATUS1: VBUS is good.
pub(super) const STATUS1_VBUS_GOOD: u8 = 1 << 5;
/// STATUS1: battery is connected.
pub(super) const STATUS1_BAT_PRESENT: u8 = 1 << 3;

/// STATUS2: VBUS input is active (low when in).
pub(super) const STATUS2_VBUS_IN: u8 = 1 << 3;
/// STATUS2: PMIC is powered on.
pub(super) const STATUS2_POWER_ON: u8 = 1 << 4;
/// STATUS2: charging state machine is in the charging state.
pub(super) const STATUS2_CHARGING: u8 = 0x01;
/// STATUS2: shift for the charging state field.
pub(super) const STATUS2_CHG_SHIFT: u8 = 5;

/// COMMON_CONFIG: power off the PMIC.
pub(super) const COMMON_POWER_OFF: u8 = 1 << 0;
/// COMMON_CONFIG: restart the PMIC and reset the SoC.
pub(super) const COMMON_RESET: u8 = 1 << 1;
/// COMMON_CONFIG: enable internal output discharge.
pub(super) const COMMON_DISCHARGE: u8 = 1 << 5;

/// DC_ONOFF_DVM: DVM ramp select.
pub(super) const DC_DVM_RAMP: u8 = 1 << 5;
/// DC_FORCE_PWM: forced-PWM bit for DCDC1 (DCDC2..4 follow).
pub(super) const DC_PWM_BASE_BIT: u8 = 2;

/// ADC_DATA high-byte masks (5 bits for VBAT, 6 bits for VBUS/VSYS/TS).
pub(super) const VBAT_HIGH_MASK: u8 = 0x1F;
/// High-byte mask for VBUS, VSYS, TS, and die temperature.
pub(super) const VOLT_HIGH_MASK: u8 = 0x3F;

/// Low-battery warning field mask (high nibble of [`LOW_BAT_WARN`]).
pub(super) const LOW_BAT_WARN_MASK: u8 = 0xF0;
/// Low-battery shutdown field mask (low nibble of [`LOW_BAT_WARN`]).
pub(super) const LOW_BAT_SHUTDOWN_MASK: u8 = 0x0F;
