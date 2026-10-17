//! CO5300 display control on the board (Phase 4.4 button wake).
//!
//! Owns the SPI2 quad transport, the panel, and the frame across the awake
//! window so the panel can be powered down before sleep and brought back after.
//!
//! The AMOLED has no dedicated rail: its power ties to VCC3V3 and its enable
//! `DSI_PWR_EN` is pulled up to ALDO2 (VL2). Power-down is therefore `SLPIN`
//! first, then `power` cuts ALDO2; because the panel loses power, wake re-runs
//! the full init and re-flushes.

use esp_hal::delay::Delay;
use esp_hal::dma::{DmaBufError, DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::{Config as SpiConfig, ConfigError, Spi};
use esp_hal::spi::{Error as SpiError, Mode};
use esp_hal::time::Rate;

use alloc::vec;
use alloc::vec::Vec;
use common::{Region, Rgb565};
use drivers::board;
use drivers::co5300::{Co5300, DisplayError, Geometry};
use hal::Display as _;
use ui::{draw_bringup as draw_bringup_pattern, MonoSurface, Painter, SurfaceError};

mod bus;

use bus::EspQspiBus;

/// SPI clock. The vendor uses 40 MHz.
const SPI_FREQUENCY_MHZ: u32 = 40;

/// DMA staging buffer and pixel chunk, in bytes (even).
const CHUNK_BYTES: usize = 2048;

/// Temporary bring-up block width in pixels.
const BRINGUP_WIDTH: u16 = 224;
/// Temporary bring-up block height in pixels.
const BRINGUP_HEIGHT: u16 = 160;

/// The display's exclusive peripherals, grouped to keep the API small.
pub struct Peripherals<'d> {
    /// SPI2, wired to the panel in quad mode.
    pub spi: esp_hal::peripherals::SPI2<'d>,
    /// DMA channel for pixel throughput.
    pub dma: esp_hal::peripherals::DMA_CH0<'d>,
    /// QSPI clock.
    pub sclk: esp_hal::peripherals::GPIO11<'d>,
    /// Chip select, driven manually so it can span chunked pixel writes.
    pub cs: esp_hal::peripherals::GPIO12<'d>,
    /// QSPI data line 0.
    pub sio0: esp_hal::peripherals::GPIO4<'d>,
    /// QSPI data line 1.
    pub sio1: esp_hal::peripherals::GPIO5<'d>,
    /// QSPI data line 2.
    pub sio2: esp_hal::peripherals::GPIO6<'d>,
    /// QSPI data line 3.
    pub sio3: esp_hal::peripherals::GPIO7<'d>,
    /// Panel reset, active low.
    pub reset: esp_hal::peripherals::GPIO8<'d>,
}

/// Errors from display bring-up.
// The payloads are surfaced through `Debug`; rustc's dead-code pass does not
// count derived formatters, so silence it here.
#[allow(dead_code)]
#[derive(Debug)]
pub enum BringUpError {
    /// SPI bus configuration failed.
    Config(ConfigError),
    /// DMA buffer construction failed.
    Dma(DmaBufError),
    /// A UI surface could not be constructed.
    Surface(SurfaceError),
    /// The driver rejected an operation.
    Display(DisplayError<SpiError>),
}

/// The board's CO5300 panel, reusable across one awake window.
pub struct Display {
    panel: Co5300<EspQspiBus<'static>, Output<'static>, Delay>,
    geometry: Geometry,
    bringup: BringUp,
}

/// Preallocated storage for the temporary bring-up pattern.
struct BringUp {
    region: Region,
    bits: Vec<u8>,
    pixels: Vec<u8>,
}

impl BringUp {
    fn new(geometry: Geometry) -> Self {
        let width = BRINGUP_WIDTH.min(geometry.width);
        let height = BRINGUP_HEIGHT.min(geometry.height);
        let region = geometry.align_region(Region::new(
            (geometry.width - width) / 2,
            (geometry.height - height) / 2,
            width,
            height,
        ));
        Self {
            region,
            bits: vec![0; MonoSurface::buffer_len(region.width, region.height)],
            pixels: vec![0; region.rgb565_len()],
        }
    }
}

impl Display {
    /// Builds the SPI2 quad transport and panel. Touches no hardware state.
    pub fn new(pins: Peripherals<'static>) -> Result<Self, BringUpError> {
        let config = SpiConfig::default()
            .with_frequency(Rate::from_mhz(SPI_FREQUENCY_MHZ))
            .with_mode(Mode::_0);

        let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) = dma_buffers!(4, CHUNK_BYTES);
        let rx = DmaRxBuf::new(rx_descriptors, rx_buffer).map_err(BringUpError::Dma)?;
        let tx = DmaTxBuf::new(tx_descriptors, tx_buffer).map_err(BringUpError::Dma)?;

        let bus = EspQspiBus {
            bus: Spi::new(pins.spi, config)
                .map_err(BringUpError::Config)?
                .with_sck(pins.sclk)
                .with_sio0(pins.sio0)
                .with_sio1(pins.sio1)
                .with_sio2(pins.sio2)
                .with_sio3(pins.sio3)
                .with_dma(pins.dma)
                .with_buffers(rx, tx),
            cs: Output::new(pins.cs, Level::High, OutputConfig::default()),
        };

        let geometry = Geometry {
            width: board::LCD_WIDTH,
            height: board::LCD_HEIGHT,
            column_offset: board::LCD_COL_OFFSET,
            row_offset: board::LCD_ROW_OFFSET,
        };
        let reset = Output::new(pins.reset, Level::High, OutputConfig::default());
        Ok(Self {
            bringup: BringUp::new(geometry),
            panel: Co5300::new(bus, reset, Delay::new(), geometry),
            geometry,
        })
    }

    /// Initializes the panel (after power-on or a power cycle) and clears it.
    pub fn init(&mut self) -> Result<(), BringUpError> {
        self.panel.init().map_err(BringUpError::Display)?;
        self.panel
            .fill_region(
                Region::full(self.geometry.width, self.geometry.height),
                Rgb565::BLACK,
            )
            .map_err(BringUpError::Display)
    }

    /// Puts the panel into sleep, before `power` cuts the enable rail.
    pub fn sleep(&mut self) -> Result<(), BringUpError> {
        self.panel.display_off().map_err(BringUpError::Display)
    }

    /// Draws the temporary UI bring-up pattern.
    // TODO(phase-5): remove once Home renders through the shell.
    pub fn draw_bringup(&mut self) -> Result<(), BringUpError> {
        let region = self.bringup.region;
        let mut bits = core::mem::take(&mut self.bringup.bits);
        let mut pixels = core::mem::take(&mut self.bringup.pixels);

        let result = match MonoSurface::new(region.width, region.height, &mut bits) {
            Ok(mut surface) => {
                {
                    let mut painter = Painter::new(&mut surface);
                    draw_bringup_pattern(&mut painter);
                }
                let needed = region.rgb565_len();
                pixels.resize(needed, 0);
                surface.expand_region(
                    Region::full(region.width, region.height),
                    &mut pixels[..needed],
                );
                self.panel
                    .write_region(region, &pixels[..needed])
                    .map_err(BringUpError::Display)
            }
            Err(error) => Err(BringUpError::Surface(error)),
        };

        self.bringup.bits = bits;
        self.bringup.pixels = pixels;
        result
    }
}
