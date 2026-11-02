//! I2S DMA transport for stereo capture and playback.

use alloc::vec::Vec;
use allocator_api2::vec::Vec as ExternalVec;
use esp_alloc::ExternalMemory;
use esp_hal::dma::DmaError;
use esp_hal::dma_circular_buffers_chunk_size;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2s::master::{
    Channels, Config as I2sConfig, ConfigError, DataFormat, Error as I2sError, I2s, I2sRx, I2sTx,
};
use esp_hal::time::Rate;
use esp_hal::Blocking;
use voice::lab::{FRAME_BYTES, SAMPLE_RATE_HZ};

mod capture;
mod playback;

/// DMA descriptor chunk size. Must be a multiple of the 4-byte stereo frame so
/// `available` never leaves a 1-3 byte tail that cannot be pushed.
const DMA_CHUNK_BYTES: usize = 4_092;
/// Internal-SRAM ring used by the RX DMA. A multiple of the chunk size so the
/// circular descriptor chain has no short tail.
pub(super) const RX_RING_BYTES: usize = DMA_CHUNK_BYTES * 2;
/// Internal-SRAM ring used by the TX DMA. A multiple of the chunk size so the
/// circular descriptor chain has no short tail.
pub(super) const TX_RING_BYTES: usize = DMA_CHUNK_BYTES * 2;

const _: () = assert!(DMA_CHUNK_BYTES % FRAME_BYTES == 0);
const _: () = assert!(RX_RING_BYTES % DMA_CHUNK_BYTES == 0);
const _: () = assert!(TX_RING_BYTES % DMA_CHUNK_BYTES == 0);
/// Capture polling interval while the ring has no data.
pub(super) const POLL_MS: u64 = 2;
/// Frames discarded after RX starts so capture begins on a settled frame.
pub(super) const PRIME_FRAMES: usize = 512;
/// Longest wait for the first primed capture data.
pub(super) const PRIME_TIMEOUT_MS: u64 = 250;
/// How often live meters are emitted.
pub(super) const METER_MS: u64 = 100;
/// Time for the amplifier to settle after a level change.
pub(super) const PA_SETTLE_MS: u64 = 5;

/// A PSRAM-backed clip.
pub type Clip = ExternalVec<u8, ExternalMemory>;

/// The audio subsystem's exclusive peripherals.
pub struct Peripherals<'d> {
    /// I2S peripheral.
    pub i2s: esp_hal::peripherals::I2S1<'d>,
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

/// Errors from the I2S transport.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum TransportError {
    /// I2S configuration was rejected.
    Config(ConfigError),
    /// The I2S peripheral rejected an operation.
    I2s(I2sError),
    /// The DMA transfer failed.
    Dma(DmaError),
    /// The clip buffer could not be allocated.
    Alloc,
    /// No frames were available.
    Empty,
}

impl From<I2sError> for TransportError {
    fn from(error: I2sError) -> Self {
        Self::I2s(error)
    }
}

impl From<DmaError> for TransportError {
    fn from(error: DmaError) -> Self {
        Self::Dma(error)
    }
}

/// The I2S endpoint owned by the audio task.
pub struct Transport {
    pub(super) tx: I2sTx<'static, Blocking>,
    pub(super) rx: I2sRx<'static, Blocking>,
    pub(super) tx_buffer: &'static mut [u8; TX_RING_BYTES],
    pub(super) rx_buffer: &'static mut [u8; RX_RING_BYTES],
    pub(super) scratch: Vec<u8>,
    pub(super) pa: Output<'static>,
}

impl Transport {
    /// Builds the stereo I2S transport with MCLK from the SoC.
    pub fn new(pins: Peripherals<'static>) -> Result<Self, TransportError> {
        let config = I2sConfig::default()
            .with_sample_rate(Rate::from_hz(SAMPLE_RATE_HZ))
            .with_channels(Channels::STEREO)
            .with_data_format(DataFormat::Data16Channel16);
        let i2s = I2s::new(pins.i2s, pins.dma, config)
            .map_err(TransportError::Config)?
            .with_mclk(pins.mclk);
        let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) =
            dma_circular_buffers_chunk_size!(RX_RING_BYTES, TX_RING_BYTES, DMA_CHUNK_BYTES);
        let tx = i2s
            .i2s_tx
            .with_bclk(pins.bclk)
            .with_ws(pins.ws)
            .with_dout(pins.dout)
            .build(tx_descriptors);
        let rx = i2s.i2s_rx.with_din(pins.din).build(rx_descriptors);
        let pa = Output::new(pins.pa, Level::Low, OutputConfig::default());
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(RX_RING_BYTES)
            .map_err(|_| TransportError::Alloc)?;
        scratch.resize(RX_RING_BYTES, 0);

        Ok(Self {
            tx,
            rx,
            tx_buffer,
            rx_buffer,
            scratch,
            pa,
        })
    }

    /// Runs `f` while TX keeps the shared clocks alive.
    pub fn with_silence<R>(&mut self, f: impl FnOnce() -> R) -> Result<R, TransportError> {
        let Self { tx, tx_buffer, .. } = self;
        tx_buffer.fill(0);
        let buffer: &[u8] = &tx_buffer[..];
        let transfer = tx.write_dma_circular(&buffer)?;
        let result = f();
        transfer.stop()?;
        Ok(result)
    }
}

/// Keeps the speaker amplifier off on every exit path.
pub(super) struct PaGuard<'a>(pub &'a mut Output<'static>);

impl Drop for PaGuard<'_> {
    fn drop(&mut self) {
        self.0.set_low();
    }
}

pub(super) fn frames_for(duration_ms: u16) -> usize {
    (u64::from(duration_ms) * u64::from(SAMPLE_RATE_HZ) / 1_000) as usize
}
