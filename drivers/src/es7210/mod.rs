//! ES7210 four-channel ADC driver.
//!
//! Configured as an I2S slave for stereo MIC1/MIC2 capture at 16 kHz with
//! 16-bit samples. The init sequence follows the vendor ES7210 user guide
//! (`5.1 Sequence - TDMIN-IIS 30DB`), which is also what the current
//! esp-bsp/ESPHome drivers use. Earlier esp_codec_dev values (0x43 analog
//! power, 0x00 mic-bias power) leave the analog blocks powered differently and
//! produce a noisy capture on this board.

use embedded_hal::i2c::I2c;
use hal::AudioCapture;

mod error;
mod regs;

#[cfg(test)]
mod tests;

pub use error::Es7210Error;

use regs::*;

/// Default microphone gain: 24 dB, matching the board vendor BSP.
const DEFAULT_GAIN_DB: u8 = 24;
/// Highest supported gain step.
const MAX_GAIN_DB: u8 = 30;
/// Register code for the default gain.
const DEFAULT_GAIN: u8 = DEFAULT_GAIN_DB / 3;

/// Registers dumped after init when firmware logging is enabled.
pub const DIAGNOSTIC_REGISTERS: [u8; 27] = [
    RESET,
    CLOCK_OFF,
    MAIN_CLOCK,
    POWER_DOWN,
    OSR,
    MODE_CONFIG,
    TIME_CONTROL0,
    TIME_CONTROL1,
    SDP_INTERFACE1,
    SDP_INTERFACE2,
    ADC34_HPF2,
    ADC34_HPF1,
    ADC12_HPF1,
    ADC12_HPF2,
    ANALOG,
    MIC12_BIAS,
    MIC34_BIAS,
    MIC1_GAIN,
    MIC2_GAIN,
    MIC3_GAIN,
    MIC4_GAIN,
    MIC12_POWER,
    MIC34_POWER,
    MIC1_POWER,
    MIC2_POWER,
    MIC3_POWER,
    MIC4_POWER,
];

/// A snapshot of the registers in [`DIAGNOSTIC_REGISTERS`].
pub type DiagnosticRegisters = [(u8, u8); DIAGNOSTIC_REGISTERS.len()];

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
        // Vendor ES7210 power-up sequence.
        self.write_reg(RESET, RESET_ALL)?;
        self.write_reg(RESET, RESET_DEFAULT)?;
        self.write_reg(TIME_CONTROL0, TIME_CYCLE)?;
        self.write_reg(TIME_CONTROL1, TIME_CYCLE)?;

        self.write_reg(ADC12_HPF2, ADC12_HPF2_VALUE)?;
        self.write_reg(ADC12_HPF1, ADC12_HPF1_VALUE)?;
        self.write_reg(ADC34_HPF2, ADC34_HPF2_VALUE)?;
        self.write_reg(ADC34_HPF1, ADC34_HPF1_VALUE)?;

        self.write_reg(SDP_INTERFACE1, SDP_16_BIT_I2S)?;
        self.write_reg(SDP_INTERFACE2, TDM_OFF)?;

        // 0xC3 powers the analog block and selects the 3.3 V VMID setup.
        self.write_reg(ANALOG, ANALOG_ON)?;
        self.write_reg(MIC12_BIAS, MIC_BIAS)?;
        self.write_reg(MIC34_BIAS, MIC_BIAS)?;

        self.write_all_gains()?;
        self.write_reg(MIC1_POWER, MIC_POWER_UP)?;
        self.write_reg(MIC2_POWER, MIC_POWER_UP)?;
        self.write_reg(MIC3_POWER, MIC_POWER_UP)?;
        self.write_reg(MIC4_POWER, MIC_POWER_UP)?;

        self.write_reg(OSR, OSR_VALUE)?;
        self.write_reg(MAIN_CLOCK, MAIN_CLOCK_VALUE)?;
        self.write_reg(POWER_DOWN, DLL_POWER_DOWN)?;

        // Only MIC1/MIC2 are used for the stereo pair. Power down MIC3/4
        // (the AEC reference pair) so its input cannot add noise or crosstalk.
        self.write_reg(MIC12_POWER, MIC_GROUP_POWER)?;
        self.write_reg(MIC34_POWER, MIC_POWER_DOWN)?;

        self.write_reg(RESET, RESET_START)?;
        self.write_reg(RESET, RESET_RELEASE)
    }

    fn configure_format(&mut self) -> Result<(), Es7210Error<I::Error>> {
        self.write_reg(SDP_INTERFACE1, SDP_16_BIT_I2S)?;
        self.write_reg(SDP_INTERFACE2, TDM_OFF)
    }

    fn write_all_gains(&mut self) -> Result<(), Es7210Error<I::Error>> {
        for reg in [MIC1_GAIN, MIC2_GAIN] {
            self.write_reg(reg, MIC_ENABLE | self.gain)?;
        }
        for reg in [MIC3_GAIN, MIC4_GAIN] {
            self.write_reg(reg, 0)?;
        }
        Ok(())
    }

    fn verify(&mut self) -> Result<(), Es7210Error<I::Error>> {
        // Keep this limited to registers the vendor driver also reads. The
        // analog/DLL state is reported by `read_diagnostics` instead of
        // blocking bring-up on parts with different readback behavior.
        self.expect(SDP_INTERFACE1, SDP_16_BIT_I2S)?;
        self.expect(SDP_INTERFACE2, TDM_OFF)?;
        for reg in [MIC1_GAIN, MIC2_GAIN] {
            let expected = MIC_ENABLE | self.gain;
            let value = self.read_reg(reg)?;
            if value != expected {
                return Err(Es7210Error::BadConfig {
                    register: reg,
                    value,
                });
            }
        }
        for reg in [MIC3_GAIN, MIC4_GAIN] {
            let value = self.read_reg(reg)?;
            if value != 0 {
                return Err(Es7210Error::BadConfig {
                    register: reg,
                    value,
                });
            }
        }
        Ok(())
    }

    /// Reads the key registers for a post-init diagnostic dump.
    pub fn read_diagnostics(&mut self) -> Result<DiagnosticRegisters, Es7210Error<I::Error>> {
        let mut out = [(0u8, 0u8); DIAGNOSTIC_REGISTERS.len()];
        for (slot, &reg) in out.iter_mut().zip(DIAGNOSTIC_REGISTERS.iter()) {
            *slot = (reg, self.read_reg(reg)?);
        }
        Ok(out)
    }

    fn expect(&mut self, reg: u8, expected: u8) -> Result<(), Es7210Error<I::Error>> {
        let value = self.read_reg(reg)?;
        if value == expected {
            Ok(())
        } else {
            Err(Es7210Error::BadConfig {
                register: reg,
                value,
            })
        }
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
}

impl<I: I2c> AudioCapture for Es7210<I> {
    type Error = Es7210Error<I::Error>;

    fn init(&mut self) -> Result<(), Self::Error> {
        self.open()?;
        // Re-apply the format after the start reset so 16-bit I2S is
        // guaranteed even if the part reloads its 24-bit default.
        self.configure_format()?;
        self.verify()
    }

    fn set_gain(&mut self, db: u8) -> Result<(), Self::Error> {
        if db > MAX_GAIN_DB || !db.is_multiple_of(3) {
            return Err(Es7210Error::InvalidGain(db));
        }
        self.gain = db / 3;
        self.write_all_gains()
    }

    fn shutdown(&mut self) -> Result<(), Self::Error> {
        // Vendor standby sequence.
        self.write_reg(MIC12_POWER, MIC_POWER_DOWN)?;
        self.write_reg(MIC34_POWER, MIC_POWER_DOWN)?;
        self.write_reg(ANALOG, ANALOG_OFF)?;
        self.write_reg(CLOCK_OFF, CLOCK_OFF_ALL)?;
        self.write_reg(POWER_DOWN, POWER_DOWN_VALUE)
    }
}
