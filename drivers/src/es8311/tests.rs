//! Host tests for the ES8311 driver against the shared register mock.

use hal::Audio;

use super::regs::*;
use super::{Es8311, Es8311Error, DEFAULT_VOLUME};
use crate::board::ES8311_I2C_ADDR;
use crate::test_support::{MockError, MockI2c};

fn chip() -> Es8311<MockI2c> {
    Es8311::new(
        MockI2c::new()
            .with_reg(CHIP_ID1, CHIP_ID1_VALUE)
            .with_reg(CLOCK_MANAGER_02, 0x07),
        ES8311_I2C_ADDR,
    )
}

#[test]
fn init_runs_reset_then_configures_playback() {
    let mut codec = chip();
    codec.init().unwrap();

    let writes = codec.i2c.writes();
    assert_eq!(
        &writes[..3],
        &[
            (RESET, RESET_ALL),
            (RESET, RESET_RELEASE),
            (RESET, POWER_ON)
        ]
    );
    assert_eq!(codec.i2c.reg(RESET), POWER_ON);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_01), CLOCK_SOURCE_MCLK);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_02), 0x07);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_03), CLOCK_03_VALUE);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_04), CLOCK_04_VALUE);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_05), CLOCK_05_VALUE);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_06), CLOCK_06_SET);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_07), CLOCK_07_SET);
    assert_eq!(codec.i2c.reg(CLOCK_MANAGER_08), CLOCK_08_VALUE);
    assert_eq!(codec.i2c.reg(SDP_IN), SDP_16_BIT_I2S);
    assert_eq!(codec.i2c.reg(SDP_OUT), SDP_16_BIT_I2S);
    assert_eq!(codec.i2c.reg(SYSTEM_0D), SYSTEM_0D_POWER_UP);
    assert_eq!(codec.i2c.reg(SYSTEM_0E), SYSTEM_0E_POWER_UP);
    assert_eq!(codec.i2c.reg(SYSTEM_12), SYSTEM_12_DAC_ON);
    assert_eq!(codec.i2c.reg(SYSTEM_13), SYSTEM_13_HP_DRIVE);
    assert_eq!(codec.i2c.reg(ADC_1C), ADC_EQ_BYPASS);
    assert_eq!(codec.i2c.reg(DAC_37), DAC_EQ_BYPASS);
    assert_eq!(codec.i2c.reg(DAC_32), DEFAULT_VOLUME);
    assert_eq!(codec.i2c.reg(DAC_31) & DAC_MUTE, 0);
}

#[test]
fn rejects_a_foreign_chip_id_before_writing() {
    let mut codec = Es8311::new(MockI2c::new(), ES8311_I2C_ADDR);
    assert_eq!(codec.init(), Err(Es8311Error::BadChipId(0)));
    assert!(codec.i2c.writes().is_empty());
}

#[test]
fn bus_failure_is_propagated() {
    let mut mock = MockI2c::new().with_reg(CHIP_ID1, CHIP_ID1_VALUE);
    mock.set_fail(true);
    let mut codec = Es8311::new(mock, ES8311_I2C_ADDR);
    assert_eq!(codec.init(), Err(Es8311Error::Bus(MockError::Nack)));
}

#[test]
fn volume_maps_to_zero_db_maximum() {
    let mut codec = chip();
    codec.init().unwrap();

    codec.set_volume(0).unwrap();
    assert_eq!(codec.i2c.reg(DAC_32), 0);
    codec.set_volume(50).unwrap();
    assert_eq!(codec.i2c.reg(DAC_32), 0x5F);
    codec.set_volume(100).unwrap();
    assert_eq!(codec.i2c.reg(DAC_32), DAC_ZERO_DB);
    assert_eq!(codec.set_volume(101), Err(Es8311Error::InvalidVolume(101)));
    assert_eq!(codec.i2c.reg(DAC_32), DAC_ZERO_DB);
}

#[test]
fn mute_and_unmute_use_the_dac_mute_bits() {
    let mut codec = chip();
    codec.init().unwrap();

    codec.mute().unwrap();
    assert_eq!(codec.i2c.reg(DAC_31) & DAC_MUTE, DAC_MUTE);
    codec.unmute().unwrap();
    assert_eq!(codec.i2c.reg(DAC_31) & DAC_MUTE, 0);
}

#[test]
fn shutdown_powers_down_the_dac_and_analog_blocks() {
    let mut codec = chip();
    codec.init().unwrap();
    codec.shutdown().unwrap();

    assert_eq!(codec.i2c.reg(DAC_32), 0);
    assert_eq!(codec.i2c.reg(DAC_31) & DAC_MUTE, DAC_MUTE);
    assert_eq!(codec.i2c.reg(SYSTEM_13), 0);
    assert_eq!(codec.i2c.reg(SYSTEM_12), SYSTEM_12_DAC_OFF);
    assert_eq!(
        codec.i2c.reg(SYSTEM_0E) & SYSTEM_0E_POWER_DOWN,
        SYSTEM_0E_POWER_DOWN
    );
    assert_eq!(codec.i2c.reg(SYSTEM_0D), SYSTEM_0D_POWER_DOWN);
    assert_eq!(codec.i2c.reg(RESET), RESET_RELEASE);
}

#[test]
fn init_after_shutdown_restores_playback() {
    let mut codec = chip();
    codec.init().unwrap();
    codec.shutdown().unwrap();
    codec.init().unwrap();

    assert_eq!(codec.i2c.reg(RESET), POWER_ON);
    assert_eq!(codec.i2c.reg(SYSTEM_12), SYSTEM_12_DAC_ON);
    assert_eq!(codec.i2c.reg(SYSTEM_13), SYSTEM_13_HP_DRIVE);
    assert_eq!(codec.i2c.reg(DAC_32), DEFAULT_VOLUME);
}
