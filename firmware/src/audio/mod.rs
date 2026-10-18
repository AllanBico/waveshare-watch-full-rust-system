//! Desk-mode audio capture and playback.
//!
//! Owns the full-duplex I2S bus, the ES8311 playback codec, the ES7210
//! microphone ADC, and the speaker-amplifier enable. The codecs are I2S
//! slaves; the ESP32-S3 drives MCLK, BCLK, and LRCK. TX keeps the shared
//! clocks alive during capture, and PA_CTRL is low except during playback.

// The loopback path is parked until voice lands in the shell.
// TODO(phase-8): remove this allow when the voice pipeline uses capture/playback.
#![allow(dead_code)]

use alloc::vec::Vec;

use allocator_api2::vec::Vec as ExternalVec;
use embassy_time::{Duration, Instant, Timer};
use esp_alloc::ExternalMemory;
use esp_hal::dma::DmaError;
use esp_hal::dma_circular_buffers;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2s::master::{
    Channels, Config as I2sConfig, ConfigError, DataFormat, Error as I2sError, I2s, I2sRx, I2sTx,
};
use esp_hal::time::Rate;
use esp_hal::Blocking;
use esp_println::println;
use hal::{Audio as _, AudioCapture as _};

use drivers::es7210::{Es7210, Es7210Error};
use drivers::es8311::{Es8311, Es8311Error, SAMPLE_RATE_HZ};

/// Length of the bring-up recording.
pub const RECORD_MS: u32 = 5_000;
/// Bytes captured per second of mono 16-bit audio.
const BYTES_PER_SECOND: usize = SAMPLE_RATE_HZ as usize * 2;
/// One recording in bytes.
const CLIP_BYTES: usize = BYTES_PER_SECOND * RECORD_MS as usize / 1000;
/// Internal-SRAM ring used by the RX DMA.
const RX_RING_BYTES: usize = 4096;
/// Internal-SRAM ring used by the TX DMA.
const TX_RING_BYTES: usize = 16_384;
/// Time for the amplifier to settle after a level change.
const PA_SETTLE_MS: u64 = 5;
/// Capture polling interval while the RX ring has no data.
const POLL_MS: u64 = 2;

const _: () = {
    assert!(CLIP_BYTES > 0);
    assert!(CLIP_BYTES.is_multiple_of(2));
};

/// The audio subsystem's exclusive peripherals.
pub struct Peripherals<'d> {
    /// I2S peripheral.
    pub i2s: esp_hal::peripherals::I2S0<'d>,
    /// DMA channel shared by playback and capture.
    pub dma: esp_hal::peripherals::DMA_CH1<'d>,
    /// I2S master clock.
    pub mclk: esp_hal::peripherals::GPIO16<'d>,
    /// I2S bit clock.
    pub bclk: esp_hal::peripherals::GPIO41<'d>,
    /// I2S word select.
    pub ws: esp_hal::peripherals::GPIO45<'d>,
    /// I2S DAC data.
    pub dout: esp_hal::peripherals::GPIO40<'d>,
    /// I2S ADC data.
    pub din: esp_hal::peripherals::GPIO42<'d>,
    /// Speaker-amplifier enable.
    pub pa: esp_hal::peripherals::GPIO46<'d>,
}

/// Errors from audio bring-up.
// The payload is surfaced through `Debug`; rustc's dead-code pass does not
// count derived formatters, so silence it here.
#[allow(dead_code)]
#[derive(Debug)]
pub enum BringUpError {
    /// I2S configuration was rejected.
    Config(ConfigError),
}

/// Errors from audio operations.
// The payloads are surfaced through `Debug`; rustc's dead-code pass does not
// count derived formatters, so silence it here.
#[allow(dead_code)]
#[derive(Debug)]
pub enum AudioError<E> {
    /// The playback codec rejected an operation.
    Codec(Es8311Error<E>),
    /// The capture codec rejected an operation.
    Capture(Es7210Error<E>),
    /// The I2S peripheral rejected an operation.
    I2s(I2sError),
    /// The DMA transfer failed.
    Dma(DmaError),
    /// The clip buffers could not be allocated.
    Alloc,
    /// The codec has not been initialized.
    NotReady,
    /// Capture produced no samples.
    CaptureEmpty,
}

impl<E> From<Es8311Error<E>> for AudioError<E> {
    fn from(error: Es8311Error<E>) -> Self {
        AudioError::Codec(error)
    }
}

impl<E> From<Es7210Error<E>> for AudioError<E> {
    fn from(error: Es7210Error<E>) -> Self {
        AudioError::Capture(error)
    }
}

/// Result of one loopback test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopbackReport {
    /// Captured bytes.
    pub bytes: usize,
    /// Largest absolute 16-bit sample.
    pub peak: u16,
}

/// Desk-mode audio endpoint.
pub struct Audio<I> {
    codec: Es8311<I>,
    mic: Es7210<I>,
    tx: I2sTx<'static, Blocking>,
    rx: I2sRx<'static, Blocking>,
    tx_buffer: &'static mut [u8; TX_RING_BYTES],
    rx_buffer: &'static mut [u8; RX_RING_BYTES],
    pa: Output<'static>,
    ready: bool,
}

impl<I: embedded_hal::i2c::I2c> Audio<I> {
    /// Builds the I2S transport and DMA rings. Touches no codec registers.
    pub fn new(
        pins: Peripherals<'static>,
        codec: Es8311<I>,
        mic: Es7210<I>,
    ) -> Result<Self, BringUpError> {
        let config = I2sConfig::default()
            .with_sample_rate(Rate::from_hz(SAMPLE_RATE_HZ))
            .with_channels(Channels::MONO)
            .with_data_format(DataFormat::Data16Channel16);
        let i2s = I2s::new(pins.i2s, pins.dma, config)
            .map_err(BringUpError::Config)?
            .with_mclk(pins.mclk);
        let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) =
            dma_circular_buffers!(RX_RING_BYTES, TX_RING_BYTES);
        let tx = i2s
            .i2s_tx
            .with_bclk(pins.bclk)
            .with_ws(pins.ws)
            .with_dout(pins.dout)
            .build(tx_descriptors);
        let rx = i2s.i2s_rx.with_din(pins.din).build(rx_descriptors);
        let pa = Output::new(pins.pa, Level::Low, OutputConfig::default());

        Ok(Self {
            codec,
            mic,
            tx,
            rx,
            tx_buffer,
            rx_buffer,
            pa,
            ready: false,
        })
    }

    /// Initializes both codecs while TX keeps the shared clocks alive.
    pub fn init(&mut self) -> Result<(), AudioError<I::Error>> {
        self.tx_buffer.fill(0);
        let buffer: &[u8] = &self.tx_buffer[..];
        let silence = self
            .tx
            .write_dma_circular(&buffer)
            .map_err(AudioError::I2s)?;
        let codec = self.codec.init().map_err(AudioError::Codec);
        let mic = match &codec {
            Ok(()) => self.mic.init().map_err(AudioError::Capture),
            Err(_) => Ok(()),
        };
        let stop = silence.stop().map_err(AudioError::Dma);
        codec?;
        mic?;
        stop?;
        self.ready = true;
        Ok(())
    }

    /// Records the test clip and immediately plays it back.
    pub async fn record_and_play(&mut self) -> Result<LoopbackReport, AudioError<I::Error>> {
        if !self.ready {
            return Err(AudioError::NotReady);
        }
        let result = self.loopback_inner().await;
        self.pa.set_low();
        let _ = self.codec.mute();
        result
    }

    /// Powers both codecs and the amplifier down.
    pub fn shutdown(&mut self) -> Result<(), AudioError<I::Error>> {
        self.ready = false;
        self.pa.set_low();
        let mic = self.mic.shutdown().map_err(AudioError::Capture);
        let codec = self.codec.shutdown().map_err(AudioError::Codec);
        mic?;
        codec
    }

    async fn loopback_inner(&mut self) -> Result<LoopbackReport, AudioError<I::Error>> {
        let mut clip = ExternalVec::new_in(ExternalMemory);
        clip.try_reserve_exact(CLIP_BYTES)
            .map_err(|_| AudioError::Alloc)?;
        clip.resize(CLIP_BYTES, 0);
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(RX_RING_BYTES)
            .map_err(|_| AudioError::Alloc)?;
        scratch.resize(RX_RING_BYTES, 0);

        println!("[audio] recording {} ms", RECORD_MS);
        let bytes = self.capture(&mut clip[..], &mut scratch).await?;
        if bytes == 0 {
            return Err(AudioError::CaptureEmpty);
        }
        let peak = peak_amplitude(&clip[..bytes]);
        println!("[audio] playing {} ms", duration_ms(bytes));
        self.play_clip(&clip[..bytes]).await?;
        Ok(LoopbackReport { bytes, peak })
    }

    async fn capture(
        &mut self,
        clip: &mut [u8],
        scratch: &mut [u8],
    ) -> Result<usize, AudioError<I::Error>> {
        self.tx_buffer.fill(0);
        let buffer: &[u8] = &self.tx_buffer[..];
        let tx_silence = self
            .tx
            .write_dma_circular(&buffer)
            .map_err(AudioError::I2s)?;
        let mut rx_transfer = self
            .rx
            .read_dma_circular(&mut self.rx_buffer)
            .map_err(AudioError::I2s)?;
        let deadline = Instant::now() + Duration::from_millis(u64::from(RECORD_MS));
        let mut written = 0;

        while written < clip.len() && Instant::now() < deadline {
            let available = rx_transfer.available().map_err(AudioError::Dma)?;
            if available == 0 {
                Timer::after(Duration::from_millis(POLL_MS)).await;
                continue;
            }
            let take = available.min(scratch.len());
            let popped = rx_transfer
                .pop(&mut scratch[..take])
                .map_err(AudioError::Dma)?;
            let copy = popped.min(clip.len() - written);
            clip[written..written + copy].copy_from_slice(&scratch[..copy]);
            written += copy;
        }

        drop(rx_transfer);
        tx_silence.stop().map_err(AudioError::Dma)?;
        Ok(written)
    }

    async fn play_clip(&mut self, clip: &[u8]) -> Result<(), AudioError<I::Error>> {
        self.codec.unmute()?;
        self.pa.set_high();
        Timer::after(Duration::from_millis(PA_SETTLE_MS)).await;
        if clip.len() <= TX_RING_BYTES {
            self.play_short(clip).await
        } else {
            self.play_streaming(clip).await
        }
    }

    async fn play_short(&mut self, clip: &[u8]) -> Result<(), AudioError<I::Error>> {
        let length = clip.len();
        self.tx_buffer[..length].copy_from_slice(clip);
        let buffer: &[u8] = &self.tx_buffer[..length];
        let transfer = self
            .tx
            .write_dma_circular(&buffer)
            .map_err(AudioError::I2s)?;
        Timer::after(Duration::from_millis(duration_ms(length))).await;
        transfer.stop().map_err(AudioError::Dma)
    }

    async fn play_streaming(&mut self, clip: &[u8]) -> Result<(), AudioError<I::Error>> {
        let first = TX_RING_BYTES.min(clip.len());
        self.tx_buffer[..first].copy_from_slice(&clip[..first]);
        let buffer: &[u8] = &self.tx_buffer[..];
        let mut transfer = self
            .tx
            .write_dma_circular(&buffer)
            .map_err(AudioError::I2s)?;
        let start = Instant::now();
        let mut offset = first;
        while offset < clip.len() {
            let available = transfer.available().map_err(AudioError::Dma)?;
            if available == 0 {
                Timer::after(Duration::from_millis(POLL_MS)).await;
                continue;
            }
            let take = available.min(clip.len() - offset);
            let pushed = transfer
                .push(&clip[offset..offset + take])
                .map_err(AudioError::Dma)?;
            offset += pushed;
        }
        let total = duration_ms(clip.len());
        let elapsed = (Instant::now() - start).as_millis();
        if elapsed < total {
            Timer::after(Duration::from_millis(total - elapsed)).await;
        }
        transfer.stop().map_err(AudioError::Dma)
    }
}

fn duration_ms(bytes: usize) -> u64 {
    bytes as u64 * 1000 / BYTES_PER_SECOND as u64
}

fn peak_amplitude(samples: &[u8]) -> u16 {
    samples
        .chunks_exact(2)
        .map(|pair| i16::from_le_bytes([pair[0], pair[1]]).unsigned_abs())
        .max()
        .unwrap_or(0)
}
