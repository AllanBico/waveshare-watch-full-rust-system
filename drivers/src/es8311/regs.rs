//! ES8311 register addresses and values used by the playback path.

pub(super) const RESET: u8 = 0x00;

pub(super) const CLOCK_MANAGER_01: u8 = 0x01;
pub(super) const CLOCK_MANAGER_02: u8 = 0x02;
pub(super) const CLOCK_MANAGER_03: u8 = 0x03;
pub(super) const CLOCK_MANAGER_04: u8 = 0x04;
pub(super) const CLOCK_MANAGER_05: u8 = 0x05;
pub(super) const CLOCK_MANAGER_06: u8 = 0x06;
pub(super) const CLOCK_MANAGER_07: u8 = 0x07;
pub(super) const CLOCK_MANAGER_08: u8 = 0x08;

pub(super) const SDP_IN: u8 = 0x09;
pub(super) const SDP_OUT: u8 = 0x0A;

pub(super) const SYSTEM_0D: u8 = 0x0D;
pub(super) const SYSTEM_0E: u8 = 0x0E;
pub(super) const SYSTEM_12: u8 = 0x12;
pub(super) const SYSTEM_13: u8 = 0x13;

pub(super) const ADC_1C: u8 = 0x1C;
pub(super) const DAC_31: u8 = 0x31;
pub(super) const DAC_32: u8 = 0x32;
pub(super) const DAC_37: u8 = 0x37;

pub(super) const CHIP_ID1: u8 = 0xFD;
pub(super) const CHIP_ID1_VALUE: u8 = 0x83;

pub(super) const RESET_ALL: u8 = 0x1F;
pub(super) const RESET_RELEASE: u8 = 0x00;
pub(super) const POWER_ON: u8 = 0x80;

pub(super) const CLOCK_SOURCE_MCLK: u8 = 0x3F;
pub(super) const CLOCK_02_CLEAR: u8 = 0xF8;
pub(super) const CLOCK_02_SET: u8 = 0x00;
pub(super) const CLOCK_03_VALUE: u8 = 0x10;
pub(super) const CLOCK_04_VALUE: u8 = 0x10;
pub(super) const CLOCK_05_VALUE: u8 = 0x00;
pub(super) const CLOCK_06_CLEAR: u8 = 0x1F;
pub(super) const CLOCK_06_SET: u8 = 0x03;
pub(super) const CLOCK_07_CLEAR: u8 = 0x3F;
pub(super) const CLOCK_07_SET: u8 = 0x00;
pub(super) const CLOCK_08_VALUE: u8 = 0xFF;

pub(super) const SDP_16_BIT_I2S: u8 = 0x0C;

pub(super) const SYSTEM_0D_POWER_UP: u8 = 0x01;
pub(super) const SYSTEM_0E_POWER_UP: u8 = 0x02;
pub(super) const SYSTEM_12_DAC_ON: u8 = 0x00;
pub(super) const SYSTEM_13_HP_DRIVE: u8 = 0x10;

pub(super) const ADC_EQ_BYPASS: u8 = 0x6A;
pub(super) const DAC_EQ_BYPASS: u8 = 0x08;
pub(super) const DAC_ZERO_DB: u8 = 0xBF;
pub(super) const DAC_MUTE: u8 = 0x60;

pub(super) const SYSTEM_12_DAC_OFF: u8 = 0x02;
pub(super) const SYSTEM_0E_POWER_DOWN: u8 = 0x60;
pub(super) const SYSTEM_0D_POWER_DOWN: u8 = 0xFC;
