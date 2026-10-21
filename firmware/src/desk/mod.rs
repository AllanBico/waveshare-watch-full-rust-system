//! Desk-only audio hardware and the Audio Lab service.
//!
//! Nothing here is initialized on a watch boot. `main` calls [`start`] only
//! after VBUS selects desk mode.

pub mod audio;
pub mod lab;

use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::i2c::master::I2c;
use esp_hal::peripherals::{DMA_CH1, GPIO16, GPIO40, GPIO41, GPIO42, GPIO45, GPIO46, I2S1};
use esp_hal::Blocking;
use esp_println::println;

use drivers::board;
use drivers::es7210::Es7210;
use drivers::es8311::Es8311;
use voice::lab::MAX_GAIN_DB;

use audio::{AudioError, Codecs, Peripherals};

/// Everything `main` hands to the desk audio bring-up.
pub struct Inputs {
    /// Task spawner.
    pub spawner: embassy_executor::Spawner,
    /// Shared I2C bus.
    pub bus: &'static core::cell::RefCell<I2c<'static, Blocking>>,
    /// I2S peripheral.
    pub i2s: I2S1<'static>,
    /// DMA channel.
    pub dma: DMA_CH1<'static>,
    /// I2S master clock.
    pub mclk: GPIO16<'static>,
    /// I2S bit clock.
    pub bclk: GPIO41<'static>,
    /// I2S word select.
    pub ws: GPIO45<'static>,
    /// I2S DAC data.
    pub dout: GPIO40<'static>,
    /// I2S ADC data.
    pub din: GPIO42<'static>,
    /// Speaker-amplifier enable.
    pub pa: GPIO46<'static>,
    /// Whether serial logging is enabled.
    pub log: bool,
}

/// The firmware-owned half of the Audio Lab.
pub struct Control {
    codecs: Codecs<RefCellDevice<'static, I2c<'static, Blocking>>>,
    log: bool,
}

/// Brings up the codecs, starts the audio task, and returns the control handle.
pub fn start(inputs: Inputs) -> Option<Control> {
    let codec = Es8311::new(RefCellDevice::new(inputs.bus), board::ES8311_I2C_ADDR);
    let mic = Es7210::new(RefCellDevice::new(inputs.bus), board::ES7210_I2C_ADDR);
    let peripherals = Peripherals {
        i2s: inputs.i2s,
        dma: inputs.dma,
        mclk: inputs.mclk,
        bclk: inputs.bclk,
        ws: inputs.ws,
        dout: inputs.dout,
        din: inputs.din,
        pa: inputs.pa,
    };
    match audio::bring_up(peripherals, codec, mic) {
        Ok(mut brought) => {
            if inputs.log {
                match brought.codecs.read_mic_diagnostics() {
                    Ok(dump) => {
                        for (reg, value) in dump {
                            println!("[audio] es7210 0x{:02X}=0x{:02X}", reg, value);
                        }
                    }
                    Err(_) => println!("[audio] es7210 diagnostic read failed"),
                }
                println!(
                    "[audio] i2s master 16 kHz/16-bit stereo, MCLK 4.096 MHz, pins mclk {} bclk {} ws {} din {}",
                    board::I2S_MCLK,
                    board::I2S_SCLK,
                    board::I2S_LRCK,
                    board::I2S_ASDOUT
                );
                println!("[audio] ES8311 + ES7210 ready");
            }
            inputs.spawner.must_spawn(lab::task(brought.transport));
            Some(Control {
                codecs: brought.codecs,
                log: inputs.log,
            })
        }
        Err(error) => {
            if inputs.log {
                println!("[audio] bring-up failed: {:?}", error);
            }
            None
        }
    }
}

impl Control {
    /// Applies a microphone gain step.
    pub fn set_gain(&mut self, db: u8) -> Result<(), lab::ControlError> {
        if db > MAX_GAIN_DB || !db.is_multiple_of(3) {
            return Err(lab::ControlError::Gain);
        }
        self.codecs.set_gain(db).map_err(|error| {
            self.log_failure("mic gain", db);
            map_codec(error)
        })
    }

    /// Applies a playback volume percentage.
    pub fn set_volume(&mut self, percent: u8) -> Result<(), lab::ControlError> {
        self.codecs.set_volume(percent).map_err(|error| {
            self.log_failure("volume", percent);
            map_codec(error)
        })
    }

    /// Mutes or unmutes playback.
    pub fn set_mute(&mut self, muted: bool) -> Result<(), lab::ControlError> {
        self.codecs.set_mute(muted).map_err(|error| {
            self.log_failure("mute", u8::from(muted));
            map_codec(error)
        })
    }

    /// Powers the codecs down after leaving desk mode.
    pub fn shutdown(&mut self) {
        lab::stop();
        if self.codecs.shutdown().is_err() {
            self.log_failure("shutdown", 0);
        }
    }

    /// Restores the codecs after returning to desk mode.
    pub fn init(&mut self) -> Result<(), lab::ControlError> {
        self.codecs.init().map_err(|error| {
            self.log_failure("init", 0);
            map_codec(error)
        })
    }

    fn log_failure(&self, operation: &str, value: u8) {
        if self.log {
            println!("[audio] {} {} failed", operation, value);
        }
    }
}

fn map_codec<E>(error: AudioError<E>) -> lab::ControlError {
    let _ = error;
    lab::ControlError::Codec
}
