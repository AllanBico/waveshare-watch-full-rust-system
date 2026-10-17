//! ES7210 register addresses and values used by the capture path.

pub(super) const RESET: u8 = 0x00;
pub(super) const CLOCK_OFF: u8 = 0x01;
pub(super) const MAIN_CLOCK: u8 = 0x02;
pub(super) const POWER_DOWN: u8 = 0x06;
pub(super) const OSR: u8 = 0x07;
pub(super) const MODE_CONFIG: u8 = 0x08;
pub(super) const TIME_CONTROL0: u8 = 0x09;
pub(super) const TIME_CONTROL1: u8 = 0x0A;
pub(super) const SDP_INTERFACE1: u8 = 0x11;
pub(super) const SDP_INTERFACE2: u8 = 0x12;

pub(super) const ADC34_HPF2: u8 = 0x20;
pub(super) const ADC34_HPF1: u8 = 0x21;
pub(super) const ADC12_HPF1: u8 = 0x22;
pub(super) const ADC12_HPF2: u8 = 0x23;

pub(super) const ANALOG: u8 = 0x40;
pub(super) const MIC12_BIAS: u8 = 0x41;
pub(super) const MIC34_BIAS: u8 = 0x42;
pub(super) const MIC1_GAIN: u8 = 0x43;
pub(super) const MIC2_GAIN: u8 = 0x44;
pub(super) const MIC3_GAIN: u8 = 0x45;
pub(super) const MIC4_GAIN: u8 = 0x46;
pub(super) const MIC1_POWER: u8 = 0x47;
pub(super) const MIC2_POWER: u8 = 0x48;
pub(super) const MIC3_POWER: u8 = 0x49;
pub(super) const MIC4_POWER: u8 = 0x4A;
pub(super) const MIC12_POWER: u8 = 0x4B;
pub(super) const MIC34_POWER: u8 = 0x4C;

pub(super) const RESET_ALL: u8 = 0xFF;
pub(super) const RESET_RELEASE: u8 = 0x41;
pub(super) const RESET_START: u8 = 0x71;

pub(super) const SLAVE_MODE_CLEAR: u8 = 0x01;
pub(super) const SLAVE_MODE_SET: u8 = 0x00;
pub(super) const CLOCK_ON: u8 = 0x3F;
pub(super) const CLOCK_ON_MIC: u8 = 0x34;
pub(super) const CLOCK_OFF_ALL: u8 = 0x7F;
pub(super) const MIC_CLOCK_MASK: u8 = 0x0B;

pub(super) const TIME_CYCLE: u8 = 0x30;
pub(super) const ADC12_HPF2_VALUE: u8 = 0x2A;
pub(super) const ADC12_HPF1_VALUE: u8 = 0x0A;
pub(super) const ADC34_HPF2_VALUE: u8 = 0x0A;
pub(super) const ADC34_HPF1_VALUE: u8 = 0x2A;

pub(super) const ANALOG_ON: u8 = 0x43;
pub(super) const ANALOG_OFF: u8 = 0xC0;
pub(super) const MIC_BIAS: u8 = 0x70;
pub(super) const OSR_VALUE: u8 = 0x20;
pub(super) const MAIN_CLOCK_VALUE: u8 = 0xC1;
pub(super) const SDP_16_BIT_I2S: u8 = 0x60;
pub(super) const TDM_OFF: u8 = 0x00;

pub(super) const MIC_POWER_UP: u8 = 0x08;
pub(super) const MIC_POWER_DOWN: u8 = 0xFF;
pub(super) const MIC_ENABLE: u8 = 0x10;
pub(super) const MIC_GAIN_MASK: u8 = 0x0F;
pub(super) const MIC_DISABLE_MASK: u8 = 0x10;
pub(super) const POWER_DOWN_VALUE: u8 = 0x07;
