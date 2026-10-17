//! ES7210 four-channel ADC driver.
//!
//! Configured as an I2S slave for mono MIC1 capture at 16 kHz with 16-bit
//! samples. The sequence follows the known vendor/esp_codec_dev driver: the
//! ES7210 datasheet in the repository has no register map, so the loopback
//! test is the verification gate.

use embedded_hal::i2c::I2c;
use hal::AudioCapture;

mod error;
mod regs;

#[cfg(test)]
mod tests;

pub use error::Es7210Error;

use regs::*;

/// Default microphone gain: 18 dB.
const DEFAULT_GAIN_DB: u8 = 18;
/// Highest supported gain step.
const MAX_GAIN_DB: u8 = 30;
/// Register code for the default gain.
const DEFAULT_GAIN: u8 = DEFAULT_GAIN_DB / 3;

/// An ES7210 ADC on a shared I2C bus.
pub struct Es7210<I> {
    i2c: I,
    address: u8,
    gain: u8,
}

impl<I: I2c> Es7210<I> {
    /// Creates a driver for the ADC at `address`. No hardware is touched
    /// until [`AudioCapture::init`].
    pub const fn new(i2c: I, address: u8) -> Self {
        Self {
            i2c,
            address,
            gain: DEFAULT_GAIN,
        }
    }

    fn open(&mut self) -> Result<(), Es7210Error<I::Error>> {
        self.write_reg(RESET, RESET_ALL)?;
        self.write_reg(RESET, RESET_RELEASE)?;
        self.write_reg(CLOCK_OFF, CLOCK_ON)?;
        self.write_reg(TIME_CONTROL0, TIME_CYCLE)?;
        self.write_reg(TIME_CONTROL1, TIME_CYCLE)?;
        self.write_reg(ADC12_HPF2, ADC12_HPF2_VALUE)?;
        self.write_reg(ADC12_HPF1, ADC12_HPF1_VALUE)?;
        self.write_reg(ADC34_HPF2, ADC34_HPF2_VALUE)?;
        self.write_reg(ADC34_HPF1, ADC34_HPF1_VALUE)?;
        self.update_reg(MODE_CONFIG, SLAVE_MODE_CLEAR, SLAVE_MODE_SET)?;
        self.write_reg(ANALOG, ANALOG_ON)?;
        self.write_reg(MIC12_BIAS, MIC_BIAS)?;
        self.write_reg(MIC34_BIAS, MIC_BIAS)?;
        self.write_reg(OSR, OSR_VALUE)?;
        self.write_reg(MAIN_CLOCK, MAIN_CLOCK_VALUE)?;
        self.select_mic1()
    }

    fn configure_format(&mut self) -> Result<(), Es7210Error<I::Error>> {
        self.write_reg(SDP_INTERFACE1, SDP_16_BIT_I2S)?;
        self.write_reg(SDP_INTERFACE2, TDM_OFF)
    }

    fn select_mic1(&mut self) -> Result<(), Es7210Error<I::Error>> {
        for reg in [MIC1_GAIN, MIC2_GAIN, MIC3_GAIN, MIC4_GAIN] {
            self.update_reg(reg, MIC_DISABLE_MASK, 0)?;
        }
        self.write_reg(MIC12_POWER, MIC_POWER_DOWN)?;
        self.write_reg(MIC34_POWER, MIC_POWER_DOWN)?;
        self.update_reg(CLOCK_OFF, MIC_CLOCK_MASK, 0)?;
        self.write_reg(MIC12_POWER, 0)?;
        self.update_reg(MIC1_GAIN, MIC_ENABLE, MIC_ENABLE)?;
        self.update_reg(MIC1_GAIN, MIC_GAIN_MASK, self.gain)
    }

    fn start(&mut self) -> Result<(), Es7210Error<I::Error>> {
        self.write_reg(CLOCK_OFF, CLOCK_ON_MIC)?;
        self.write_reg(POWER_DOWN, 0)?;
        self.write_reg(ANALOG, ANALOG_ON)?;
        self.write_reg(MIC1_POWER, MIC_POWER_UP)?;
        self.write_reg(MIC2_POWER, MIC_POWER_UP)?;
        self.write_reg(MIC3_POWER, MIC_POWER_UP)?;
        self.write_reg(MIC4_POWER, MIC_POWER_UP)?;
        self.select_mic1()?;
        self.write_reg(ANALOG, ANALOG_ON)?;
        self.write_reg(RESET, RESET_START)?;
        self.write_reg(RESET, RESET_RELEASE)
    }

    fn read_reg(&mut self, reg: u8) -> Result<u8, Es7210Error<I::Error>> {
        let mut value = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut value)
            .map_err(Es7210Error::Bus)?;
        Ok(value[0])
    }

    fn write_reg(&mut self, reg: u8, value: u8) -> Result<(), Es7210Error<I::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(Es7210Error::Bus)
    }

    fn update_reg(&mut self, reg: u8, clear: u8, set: u8) -> Result<(), Es7210Error<I::Error>> {
        let value = (self.read_reg(reg)? & !clear) | (set & clear);
        self.write_reg(reg, value)
    }
}

impl<I: I2c> AudioCapture for Es7210<I> {
    type Error = Es7210Error<I::Error>;

    fn init(&mut self) -> Result<(), Self::Error> {
        self.open()?;
        self.configure_format()?;
        self.start()
    }

    fn set_gain(&mut self, db: u8) -> Result<(), Self::Error> {
        if db > MAX_GAIN_DB || !db.is_multiple_of(3) {
            return Err(Es7210Error::InvalidGain(db));
        }
        self.gain = db / 3;
        self.update_reg(MIC1_GAIN, MIC_GAIN_MASK, self.gain)
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        self.write_reg(MIC1_POWER, MIC_POWER_DOWN)?;
        self.write_reg(MIC2_POWER, MIC_POWER_DOWN)?;
        self.write_reg(MIC3_POWER, MIC_POWER_DOWN)?;
        self.write_reg(MIC4_POWER, MIC_POWER_DOWN)?;
        self.write_reg(MIC12_POWER, MIC_POWER_DOWN)?;
        self.write_reg(MIC34_POWER, MIC_POWER_DOWN)?;
        self.write_reg(ANALOG, ANALOG_OFF)?;
        self.write_reg(CLOCK_OFF, CLOCK_OFF_ALL)?;
        self.write_reg(POWER_DOWN, POWER_DOWN_VALUE)
    }
}
