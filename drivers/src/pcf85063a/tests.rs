//! Host tests for the PCF85063A driver against the shared register mock.

use common::DateTime;
use hal::Time;

use super::{Pcf85063a, RtcError};
use crate::board::RTC_I2C_ADDR;
use crate::test_support::MockI2c;

const SAMPLE: DateTime = DateTime::new(2026, 10, 17, 12, 34, 56);

fn scripted() -> MockI2c {
    MockI2c::new()
        .with_reg(super::regs::SECONDS, 0x56)
        .with_reg(super::regs::SECONDS + 1, 0x34)
        .with_reg(super::regs::SECONDS + 2, 0x12)
        .with_reg(super::regs::SECONDS + 3, 0x17)
        .with_reg(super::regs::SECONDS + 4, 0x06)
        .with_reg(super::regs::SECONDS + 5, 0x10)
        .with_reg(super::regs::SECONDS + 6, 0x26)
}

#[test]
fn init_forces_24h_and_disables_clkout() {
    let mut rtc = Pcf85063a::new(MockI2c::new(), RTC_I2C_ADDR);
    rtc.init().unwrap();
    let ctrl1 = rtc.i2c.reg(super::regs::CTRL1);
    assert_eq!(
        ctrl1 & (super::regs::CTRL1_STOP | super::regs::CTRL1_12_HOUR),
        0
    );
    assert_eq!(
        rtc.i2c.reg(super::regs::CTRL2) & super::regs::CTRL2_COF_MASK,
        super::regs::CTRL2_COF_OFF
    );
}

#[test]
fn init_preserves_the_time_registers() {
    let mock = scripted();
    let mut rtc = Pcf85063a::new(mock, RTC_I2C_ADDR);
    rtc.init().unwrap();
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS), 0x56);
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 2), 0x12);
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 6), 0x26);
}

#[test]
fn reads_bcd_datetime() {
    let mut rtc = Pcf85063a::new(scripted(), RTC_I2C_ADDR);
    assert_eq!(rtc.read_datetime().unwrap(), SAMPLE);
}

#[test]
fn oscillator_stop_is_invalid() {
    let mock = scripted().with_reg(super::regs::SECONDS, 0xD6);
    let mut rtc = Pcf85063a::new(mock, RTC_I2C_ADDR);
    assert_eq!(rtc.read_datetime().unwrap_err(), RtcError::TimeInvalid);
}

#[test]
fn set_writes_block_and_weekday() {
    let mut rtc = Pcf85063a::new(MockI2c::new(), RTC_I2C_ADDR);
    rtc.set_datetime(&SAMPLE).unwrap();
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS), 0x56);
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 1), 0x34);
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 2), 0x12);
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 3), 0x17);
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 4), SAMPLE.weekday());
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 5), 0x10);
    assert_eq!(rtc.i2c.reg(super::regs::SECONDS + 6), 0x26);
}

#[test]
fn invalid_fields_are_rejected() {
    let mut rtc = Pcf85063a::new(MockI2c::new(), RTC_I2C_ADDR);
    let bad = DateTime::new(2026, 13, 1, 0, 0, 0);
    assert_eq!(rtc.set_datetime(&bad).unwrap_err(), RtcError::InvalidField);
    assert!(rtc.i2c.writes().is_empty());
}

#[test]
fn time_trait_roundtrips_epoch() {
    let mut rtc = Pcf85063a::new(scripted(), RTC_I2C_ADDR);
    let expected = SAMPLE.to_timestamp().unwrap();
    assert_eq!(rtc.now().unwrap(), expected);

    let other = DateTime::new(2000, 1, 1, 0, 0, 0).to_timestamp().unwrap();
    rtc.set(other).unwrap();
    assert_eq!(rtc.now().unwrap(), other);
}
