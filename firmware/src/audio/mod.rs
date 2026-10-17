//! Desk-mode audio playback.
//!
//! Owns the I2S transmitter, the ES8311 codec, and the speaker-amplifier
//! enable. The codec is an I2S slave; the ESP32-S3 drives MCLK, BCLK, and
//! LRCK. PA_CTRL is kept low except while a tone is playing.

use alloc::vec::Vec;

use embassy_time::{Duration, Timer};
use esp_hal::dma::DmaError;
use esp_hal::dma_circular_descriptors;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2s::master::{
    Channels, Config as I2sConfig, ConfigError, DataFormat, Error as I2sError, I2s, I2sTx,
};
use esp_hal::time::Rate;
use esp_hal::Blocking;
use hal::Audio as _;

use drivers::es8311::{Es8311, Es8311Error, SAMPLE_RATE_HZ};

/// Frequency of the bring-up tone.
const TONE_FREQ_HZ: u32 = 1_000;
/// Peak sample of the bring-up tone, about -18 dBFS.
const TONE_AMPLITUDE: i16 = 4_000;
/// Length of one DMA buffer of tone data.
const TONE_CHUNK_MS: u32 = 250;
/// Total length of the bring-up tone.
const TEST_TONE_MS: u32 = 1_000;
/// Time for the amplifier to settle after a level change.
const PA_SETTLE_MS: u64 = 5;
/// Samples in one DMA buffer.
const TONE_SAMPLES: usize = (SAMPLE_RATE_HZ as usize * TONE_CHUNK_MS as usize) / 1000;
/// Bytes in one mono 16-bit DMA buffer.
const TONE_CHUNK_BYTES: usize = TONE_SAMPLES * 2;

/// One period of a 1 kHz sine at 16 kHz, as signed 16-bit samples.
const SINE_16: [i16; 16] = [
    0,
    sine(3827),
    sine(7071),
    sine(9239),
    TONE_AMPLITUDE,
    sine(9239),
    sine(7071),
    sine(3827),
    0,
    sine(-3827),
    sine(-7071),
    sine(-9239),
    -TONE_AMPLITUDE,
    sine(-9239),
    sine(-7071),
    sine(-3827),
];

const fn sine(ratio: i32) -> i16 {
    (TONE_AMPLITUDE as i32 * ratio / 10_000) as i16
}

const _: () = {
    assert!(SAMPLE_RATE_HZ / TONE_FREQ_HZ == SINE_16.len() as u32);
    assert!(TONE_SAMPLES.is_multiple_of(SINE_16.len()));
};

/// The audio subsystem's exclusive peripherals.
pub struct Peripherals<'d> {
    /// I2S peripheral.
    pub i2s: esp_hal::peripherals::I2S0<'d>,
    /// DMA channel for playback.
    pub dma: esp_hal::peripherals::DMA_CH1<'d>,
    /// I2S master clock.
    pub mclk: esp_hal::peripherals::GPIO16<'d>,
    /// I2S bit clock.
    pub bclk: esp_hal::peripherals::GPIO41<'d>,
    /// I2S word select.
    pub ws: esp_hal::peripherals::GPIO45<'d>,
    /// I2S DAC data.
    pub dout: esp_hal::peripherals::GPIO40<'d>,
    /// Speaker-amplifier enable.
    pub pa: esp_hal::peripherals::GPIO46<'d>,
}

/// Errors from audio bring-up.
// The payloads are surfaced through `Debug`; rustc's dead-code pass does not
// count derived formatters, so silence it here.
#[allow(dead_code)]
#[derive(Debug)]
pub enum BringUpError {
    /// I2S configuration was rejected.
    Config(ConfigError),
    /// The tone buffer could not be allocated.
    Alloc,
}

/// Errors from audio operations.
// The payloads are surfaced through `Debug`; rustc's dead-code pass does not
// count derived formatters, so silence it here.
#[allow(dead_code)]
#[derive(Debug)]
pub enum AudioError<E> {
    /// The codec rejected an operation.
    Codec(Es8311Error<E>),
    /// The I2S peripheral rejected an operation.
    I2s(I2sError),
    /// The DMA transfer failed.
    Dma(DmaError),
    /// The codec has not been initialized.
    NotReady,
}

impl<E> From<Es8311Error<E>> for AudioError<E> {
    fn from(error: Es8311Error<E>) -> Self {
        AudioError::Codec(error)
    }
}

/// Desk-mode playback path.
pub struct Audio<I> {
    codec: Es8311<I>,
    tx: I2sTx<'static, Blocking>,
    pa: Output<'static>,
    tone: Vec<u8>,
    ready: bool,
}

impl<I: embedded_hal::i2c::I2c> Audio<I> {
    /// Builds the I2S transport and tone buffer. Touches no codec registers.
    pub fn new(pins: Peripherals<'static>, codec: Es8311<I>) -> Result<Self, BringUpError> {
        let config = I2sConfig::default()
            .with_sample_rate(Rate::from_hz(SAMPLE_RATE_HZ))
            .with_channels(Channels::MONO)
            .with_data_format(DataFormat::Data16Channel16);
        let i2s = I2s::new(pins.i2s, pins.dma, config)
            .map_err(BringUpError::Config)?
            .with_mclk(pins.mclk);
        let (_rx_descriptors, tx_descriptors) = dma_circular_descriptors!(0, TONE_CHUNK_BYTES);
        let tx = i2s
            .i2s_tx
            .with_bclk(pins.bclk)
            .with_ws(pins.ws)
            .with_dout(pins.dout)
            .build(tx_descriptors);

        let pa = Output::new(pins.pa, Level::Low, OutputConfig::default());
        let mut tone = Vec::new();
        tone.try_reserve_exact(TONE_CHUNK_BYTES)
            .map_err(|_| BringUpError::Alloc)?;
        tone.resize(TONE_CHUNK_BYTES, 0);

        Ok(Self {
            codec,
            tx,
            pa,
            tone,
            ready: false,
        })
    }

    /// Initializes the codec while keeping MCLK alive for its power-up state
    /// machine.
    pub fn init(&mut self) -> Result<(), AudioError<I::Error>> {
        self.tone.fill(0);
        let buffer = self.tone.as_slice();
        let silence = self
            .tx
            .write_dma_circular(&buffer)
            .map_err(AudioError::I2s)?;
        let init = self.codec.init().map_err(AudioError::Codec);
        let stop = silence.stop().map_err(AudioError::Dma);
        init?;
        stop?;
        fill_tone(&mut self.tone);
        self.ready = true;
        Ok(())
    }

    /// Plays the one-second bring-up tone.
    pub async fn play_test_tone(&mut self) -> Result<(), AudioError<I::Error>> {
        if !self.ready {
            return Err(AudioError::NotReady);
        }
        let result = self.play_tone().await;
        self.pa.set_low();
        let _ = self.codec.mute();
        result
    }

    /// Powers the codec and amplifier down.
    pub fn shutdown(&mut self) -> Result<(), AudioError<I::Error>> {
        self.ready = false;
        self.pa.set_low();
        self.codec.shutdown().map_err(AudioError::Codec)
    }

    async fn play_tone(&mut self) -> Result<(), AudioError<I::Error>> {
        self.codec.unmute()?;
        self.pa.set_high();
        Timer::after(Duration::from_millis(PA_SETTLE_MS)).await;
        let buffer = self.tone.as_slice();
        let transfer = self
            .tx
            .write_dma_circular(&buffer)
            .map_err(AudioError::I2s)?;
        Timer::after(Duration::from_millis(u64::from(TEST_TONE_MS))).await;
        transfer.stop().map_err(AudioError::Dma)
    }
}

fn fill_tone(buffer: &mut [u8]) {
    for (index, sample) in buffer.chunks_exact_mut(2).enumerate() {
        let bytes = SINE_16[index % SINE_16.len()].to_le_bytes();
        sample[0] = bytes[0];
        sample[1] = bytes[1];
    }
}
