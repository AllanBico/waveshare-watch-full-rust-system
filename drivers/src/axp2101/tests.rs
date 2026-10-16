//! Host tests for the AXP2101 driver against a register-file mock.

use crate::test_support::{MockError, MockI2c};

use super::regs;
use super::{
    AdcChannel, Axp2101, AxpError, ChargeState, ChargeTarget, DcdcMode, DvmRamp, IrqMask,
    LongPressAction, Millivolts, Percent, Rail, RailError,
};
use crate::board::PMIC_I2C_ADDR;

fn probe() -> Axp2101<MockI2c> {
    let mock = MockI2c::new().with_reg(regs::IC_TYPE, 0x4A);
    Axp2101::new(mock, PMIC_I2C_ADDR).expect("probe succeeds")
}

#[test]
fn probe_accepts_axp2101() {
    let mut pmic = probe();
    assert_eq!(pmic.read_chip_id().unwrap(), 0x4A);
}

#[test]
fn probe_rejects_wrong_id() {
    let mock = MockI2c::new().with_reg(regs::IC_TYPE, 0x00);
    assert_eq!(
        Axp2101::new(mock, PMIC_I2C_ADDR).unwrap_err(),
        AxpError::BadChipId(0)
    );
}

#[test]
fn bus_error_maps_to_bus() {
    let mut pmic = probe();
    pmic.i2c.set_fail(true);
    assert_eq!(
        pmic.read_battery_millivolts().unwrap_err(),
        AxpError::Bus(MockError::Nack)
    );
}

#[test]
fn reads_battery_voltage() {
    let mock = MockI2c::new()
        .with_reg(regs::IC_TYPE, 0x4A)
        .with_reg(regs::VBAT_H, 0x0F)
        .with_reg(regs::VBAT_H + 1, 0xA0);
    let mut pmic = Axp2101::new(mock, PMIC_I2C_ADDR).expect("probe");
    assert_eq!(
        pmic.read_battery_millivolts().unwrap(),
        Millivolts::new(4000)
    );
}

#[test]
fn reads_consistent_snapshot() {
    let mock = MockI2c::new()
        .with_reg(regs::IC_TYPE, 0x4A)
        .with_reg(regs::STATUS1, 0x28)
        .with_reg(regs::STATUS1 + 1, 0x22)
        .with_reg(regs::VBAT_H, 0x0F)
        .with_reg(regs::VBAT_H + 1, 0xA0)
        .with_reg(regs::VBAT_H + 4, 0x13)
        .with_reg(regs::VBAT_H + 5, 0x88)
        .with_reg(regs::VBAT_H + 6, 0x0F)
        .with_reg(regs::VBAT_H + 7, 0x0A)
        .with_reg(regs::BAT_PERCENT, 85);
    let mut pmic = Axp2101::new(mock, PMIC_I2C_ADDR).expect("probe");
    let snap = pmic.read_power_snapshot().unwrap();
    assert_eq!(snap.battery, Millivolts::new(4000));
    assert_eq!(snap.vbus, Millivolts::new(5000));
    assert_eq!(snap.system, Millivolts::new(3850));
    assert_eq!(snap.percent, Percent::new(85));
    assert!(snap.status.charging);
    assert_eq!(snap.status.charge_state, Some(ChargeState::Cc));
}

#[test]
fn snapshot_is_all_or_nothing() {
    let mut pmic = probe();
    pmic.i2c.set_fail(true);
    assert!(matches!(
        pmic.read_power_snapshot().unwrap_err(),
        AxpError::Bus(MockError::Nack)
    ));
}

#[test]
fn reads_wake_power_state() {
    let mock = MockI2c::new()
        .with_reg(regs::IC_TYPE, 0x4A)
        .with_reg(regs::STATUS1, 0x20)
        .with_reg(regs::VBAT_H, 0x0F)
        .with_reg(regs::VBAT_H + 1, 0xA0);
    let mut pmic = Axp2101::new(mock, PMIC_I2C_ADDR).expect("probe");
    let state = pmic.read_wake_power_state().unwrap();
    assert!(state.vbus_present);
    assert_eq!(state.battery, Millivolts::new(4000));
}

#[test]
fn set_rail_voltage_preserves_work_mode_bit() {
    let mock = MockI2c::new()
        .with_reg(regs::IC_TYPE, 0x4A)
        .with_reg(regs::LDO_VOL_BASE, 0x80);
    let mut pmic = Axp2101::new(mock, PMIC_I2C_ADDR).expect("probe");
    pmic.set_rail_voltage(Rail::Aldo1, Millivolts::new(1800))
        .unwrap();
    assert_eq!(pmic.i2c.reg(regs::LDO_VOL_BASE), 0x80 | 13);
}

#[test]
fn rail_voltage_rejects_bad_values() {
    let mut pmic = probe();
    assert_eq!(
        pmic.set_rail_voltage(Rail::Dcdc1, Millivolts::new(1505)),
        Err(AxpError::Voltage(RailError::OffStep))
    );
    assert_eq!(
        pmic.set_rail_voltage(Rail::Dcdc1, Millivolts::new(1000)),
        Err(AxpError::Voltage(RailError::OutOfRange))
    );
}

#[test]
fn enable_and_read_rail() {
    let mut pmic = probe();
    pmic.enable_rail(Rail::Aldo3, true).unwrap();
    assert_eq!(pmic.i2c.reg(regs::LDO_ONOFF0), 0x04);
    assert!(pmic.is_rail_enabled(Rail::Aldo3).unwrap());
    pmic.enable_rail(Rail::Aldo3, false).unwrap();
    assert!(!pmic.is_rail_enabled(Rail::Aldo3).unwrap());
}

#[test]
fn adc_channels_are_independent() {
    let mut pmic = probe();
    pmic.set_adc_channel(AdcChannel::BatteryVoltage, true)
        .unwrap();
    pmic.set_adc_channel(AdcChannel::DieTemp, true).unwrap();
    assert_eq!(pmic.i2c.reg(regs::ADC_ENABLE), 0x11);
}

#[test]
fn battery_detection_toggle() {
    let mut pmic = probe();
    pmic.set_battery_detection(true).unwrap();
    assert_eq!(pmic.i2c.reg(regs::BAT_DETECT), 0x01);
}

#[test]
fn charge_target_written() {
    let mut pmic = probe();
    pmic.set_charge_target(ChargeTarget::V4_2).unwrap();
    assert_eq!(pmic.i2c.reg(regs::CV_CHG_VOL), 0x02);
}

#[test]
fn low_battery_thresholds_share_register() {
    let mut pmic = probe();
    pmic.set_low_battery_warning(Percent::new(12)).unwrap();
    assert_eq!(pmic.i2c.reg(regs::LOW_BAT_WARN) & 0xF0, 0x70);
    pmic.set_low_battery_shutdown(Percent::new(10)).unwrap();
    assert_eq!(pmic.i2c.reg(regs::LOW_BAT_WARN) & 0x0F, 0x0A);
    assert_eq!(pmic.i2c.reg(regs::LOW_BAT_WARN) & 0xF0, 0x70);
}

#[test]
fn system_shutdown_voltage_validates() {
    let mut pmic = probe();
    pmic.set_system_shutdown_voltage(Millivolts::new(3100))
        .unwrap();
    assert_eq!(pmic.i2c.reg(regs::VOFF), 5);
    assert_eq!(
        pmic.set_system_shutdown_voltage(Millivolts::new(3150)),
        Err(AxpError::Voltage(RailError::OffStep))
    );
    assert_eq!(
        pmic.set_system_shutdown_voltage(Millivolts::new(2400)),
        Err(AxpError::Voltage(RailError::OutOfRange))
    );
}

#[test]
fn dcdc_mode_only_for_dcdc_rails() {
    let mut pmic = probe();
    pmic.set_dcdc_mode(Rail::Dcdc1, DcdcMode::Pwm).unwrap();
    assert_eq!(pmic.i2c.reg(regs::DC_FORCE_PWM), 0x04);
    assert_eq!(
        pmic.set_dcdc_mode(Rail::Aldo1, DcdcMode::Pwm),
        Err(AxpError::Voltage(RailError::NotApplicable))
    );
}

#[test]
fn discharge_and_dvm_settings() {
    let mut pmic = probe();
    pmic.set_internal_discharge(true).unwrap();
    assert_eq!(pmic.i2c.reg(regs::COMMON_CONFIG) & 0x20, 0x20);
    pmic.set_dvm_ramp(DvmRamp::Us31_250).unwrap();
    assert_eq!(pmic.i2c.reg(regs::DC_ONOFF_DVM) & 0x20, 0x20);
}

#[test]
fn reads_die_temperature() {
    let mock = MockI2c::new()
        .with_reg(regs::IC_TYPE, 0x4A)
        .with_reg(regs::DIE_TEMP_H, 0x01)
        .with_reg(regs::DIE_TEMP_H + 1, 0x02);
    let mut pmic = Axp2101::new(mock, PMIC_I2C_ADDR).expect("probe");
    assert_eq!(pmic.read_die_temperature().unwrap(), 0x0102);
}

#[test]
fn pwron_long_press_configures() {
    let mut pmic = probe();
    pmic.set_pwron_long_press(LongPressAction::Restart).unwrap();
    assert_eq!(pmic.i2c.reg(regs::PWROFF_EN), 0x03);
}

#[test]
fn irq_enable_clear_and_pending() {
    let mut pmic = probe();
    pmic.set_irq_enabled(IrqMask::VBUS_INSERT | IrqMask::CHARGE_DONE, true)
        .unwrap();
    assert_eq!(pmic.i2c.reg(regs::IRQ_ENABLE0 + 1), 1 << 7);
    assert_eq!(pmic.i2c.reg(regs::IRQ_ENABLE0 + 2), 1 << 4);

    pmic.set_irq_enabled(IrqMask::VBUS_INSERT, false).unwrap();
    assert_eq!(pmic.i2c.reg(regs::IRQ_ENABLE0 + 1), 0);

    pmic.clear_irq(IrqMask::VBUS_INSERT).unwrap();
    assert_eq!(pmic.i2c.reg(regs::IRQ_STATUS0 + 1), 1 << 7);

    pmic.i2c.set_reg(regs::IRQ_STATUS0 + 2, 1 << 4);
    let pending = pmic.pending_irq().unwrap();
    assert_eq!(
        pending.bits() & IrqMask::CHARGE_DONE.bits(),
        IrqMask::CHARGE_DONE.bits()
    );
}

#[test]
fn records_bus_writes() {
    let mut pmic = probe();
    pmic.set_battery_detection(true).unwrap();
    assert!(pmic.i2c.writes().contains(&(regs::BAT_DETECT, 0x01)));
}

#[test]
fn power_off_and_reset() {
    let mut pmic = probe();
    pmic.power_off().unwrap();
    assert_eq!(pmic.i2c.reg(regs::COMMON_CONFIG), 0x01);
    pmic.reset_pmic().unwrap();
    assert_eq!(pmic.i2c.reg(regs::COMMON_CONFIG), 0x03);
}
