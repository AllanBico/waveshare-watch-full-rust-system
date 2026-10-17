//! Host tests for the ES7210 driver against the shared register mock.

use hal::AudioCapture;

use super::regs::*;
use super::{Es7210, Es7210Error, DEFAULT_GAIN};
use crate::board::ES7210_I2C_ADDR;
use crate::test_support::{MockError, MockI2c};

fn adc() -> Es7210<MockI2c> {
    Es7210::new(MockI2c::new(), ES7210_I2C_ADDR)
}

#[test]
fn init_runs_the_vendor_sequence_and_enables_mic1() {
    let mut adc = adc();
    adc.init().unwrap();

    let writes = adc.i2c.writes();
    assert_eq!(&writes[..2], &[(RESET, RESET_ALL), (RESET, RESET_RELEASE)]);
    assert_eq!(adc.i2c.reg(MODE_CONFIG) & SLAVE_MODE_CLEAR, 0);
    assert_eq!(adc.i2c.reg(SDP_INTERFACE1), SDP_16_BIT_I2S);
    assert_eq!(adc.i2c.reg(SDP_INTERFACE2), TDM_OFF);
    assert_eq!(adc.i2c.reg(ANALOG), ANALOG_ON);
    assert_eq!(adc.i2c.reg(MIC12_BIAS), MIC_BIAS);
    assert_eq!(adc.i2c.reg(MIC34_BIAS), MIC_BIAS);
    assert_eq!(adc.i2c.reg(OSR), OSR_VALUE);
    assert_eq!(adc.i2c.reg(MAIN_CLOCK), MAIN_CLOCK_VALUE);
    assert_eq!(adc.i2c.reg(MIC1_GAIN) & MIC_ENABLE, MIC_ENABLE);
    assert_eq!(adc.i2c.reg(MIC1_GAIN) & MIC_GAIN_MASK, DEFAULT_GAIN);
    assert_eq!(adc.i2c.reg(MIC12_POWER), 0);
    assert_eq!(adc.i2c.reg(MIC34_POWER), MIC_POWER_DOWN);
    assert_eq!(adc.i2c.reg(MIC1_POWER), MIC_POWER_UP);
    assert_eq!(adc.i2c.reg(MIC2_POWER), MIC_POWER_UP);
    assert_eq!(adc.i2c.reg(MIC3_POWER), MIC_POWER_UP);
    assert_eq!(adc.i2c.reg(MIC4_POWER), MIC_POWER_UP);
    assert_eq!(adc.i2c.reg(CLOCK_OFF), CLOCK_ON_MIC);
    assert_eq!(adc.i2c.reg(RESET), RESET_RELEASE);
    assert_eq!(adc.i2c.reg(POWER_DOWN), 0);
}

#[test]
fn gain_accepts_only_three_db_steps() {
    let mut adc = adc();
    adc.init().unwrap();

    adc.set_gain(0).unwrap();
    assert_eq!(adc.i2c.reg(MIC1_GAIN) & MIC_GAIN_MASK, 0);
    adc.set_gain(30).unwrap();
    assert_eq!(adc.i2c.reg(MIC1_GAIN) & MIC_GAIN_MASK, DEFAULT_GAIN);
    assert_eq!(adc.set_gain(31), Err(Es7210Error::InvalidGain(31)));
    assert_eq!(adc.set_gain(4), Err(Es7210Error::InvalidGain(4)));
    assert_eq!(adc.i2c.reg(MIC1_GAIN) & MIC_GAIN_MASK, DEFAULT_GAIN);
}

#[test]
fn shutdown_powers_down_mics_and_clocks() {
    let mut adc = adc();
    adc.init().unwrap();
    adc.shutdown().unwrap();

    assert_eq!(adc.i2c.reg(MIC1_POWER), MIC_POWER_DOWN);
    assert_eq!(adc.i2c.reg(MIC2_POWER), MIC_POWER_DOWN);
    assert_eq!(adc.i2c.reg(MIC3_POWER), MIC_POWER_DOWN);
    assert_eq!(adc.i2c.reg(MIC4_POWER), MIC_POWER_DOWN);
    assert_eq!(adc.i2c.reg(MIC12_POWER), MIC_POWER_DOWN);
    assert_eq!(adc.i2c.reg(MIC34_POWER), MIC_POWER_DOWN);
    assert_eq!(adc.i2c.reg(ANALOG), ANALOG_OFF);
    assert_eq!(adc.i2c.reg(CLOCK_OFF), CLOCK_OFF_ALL);
    assert_eq!(adc.i2c.reg(POWER_DOWN), POWER_DOWN_VALUE);
}

#[test]
fn init_after_shutdown_restores_capture() {
    let mut adc = adc();
    adc.init().unwrap();
    adc.shutdown().unwrap();
    adc.init().unwrap();

    assert_eq!(adc.i2c.reg(ANALOG), ANALOG_ON);
    assert_eq!(adc.i2c.reg(MIC1_POWER), MIC_POWER_UP);
    assert_eq!(adc.i2c.reg(MIC12_POWER), 0);
    assert_eq!(adc.i2c.reg(RESET), RESET_RELEASE);
}

#[test]
fn bus_failure_is_propagated() {
    let mut mock = MockI2c::new();
    mock.set_fail(true);
    let mut adc = Es7210::new(mock, ES7210_I2C_ADDR);
    assert_eq!(adc.init(), Err(Es7210Error::Bus(MockError::Nack)));
}
