//! CO5300 display control on the board (Phase 3.4).
//!
//! Owns the SPI2 quad transport, the panel, and the frame across the awake
//! window so the panel can be powered down before sleep and brought back after.
//!
//! The AMOLED has no dedicated rail: its power ties to VCC3V3 and its enable
//! `DSI_PWR_EN` is pulled up to ALDO2 (VL2). Power-down is therefore `SLPIN`
//! first, then `power` cuts ALDO2; because the panel loses power, wake re-runs
//! the full init and re-flushes.

use alloc::vec;

use esp_hal::delay::Delay;
use esp_hal::dma::{DmaBufError, DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::{Config as SpiConfig, ConfigError, Spi};
use esp_hal::spi::{Error as SpiError, Mode};
use esp_hal::time::Rate;

use common::{Region, Rgb565};
use drivers::board;
use drivers::co5300::{Co5300, DisplayError, Geometry};
use hal::Display as _;
use ui::{clock_size, digits_size, draw_digits, draw_time, MonoBuffer};

mod bus;

use bus::EspQspiBus;

/// SPI clock. The vendor uses 40 MHz.
const SPI_FREQUENCY_MHZ: u32 = 40;

/// DMA staging buffer and pixel chunk, in bytes (even).
const CHUNK_BYTES: usize = 2048;

/// Integer scale for the clock line.
const CLOCK_SCALE: u16 = 3;

/// Integer scale for the battery line.
const BATTERY_SCALE: u16 = 2;

/// Vertical gap between the clock and battery lines, in pixels.
const LINE_GAP: u16 = 8;

/// Battery readout width, in decimal digits (millivolts).
const BATTERY_DIGITS: u8 = 4;

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
    /// The driver rejected an operation.
    Display(DisplayError<SpiError>),
}

/// The board's CO5300 panel, reusable across one awake window.
pub struct Display {
    panel: Co5300<EspQspiBus<'static>, Output<'static>, Delay>,
    geometry: Geometry,
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

    /// Redraws the clock and battery readout. Call after [`Self::init`].
    pub fn draw(
        &mut self,
        hour: u8,
        minute: u8,
        second: u8,
        battery_mv: u16,
    ) -> Result<(), BringUpError> {
        let mut frame = MonoBuffer::new(self.geometry.width, self.geometry.height);

        let (clock_width, clock_height) = clock_size(CLOCK_SCALE);
        let (battery_width, battery_height) = digits_size(BATTERY_DIGITS, BATTERY_SCALE);
        let block_width = clock_width.max(battery_width);
        let block_height = clock_height + LINE_GAP + battery_height;
        let block_x = (self.geometry.width - block_width) / 2;
        let block_y = (self.geometry.height - block_height) / 2;

        draw_time(
            &mut frame,
            hour,
            minute,
            second,
            (block_x + (block_width - clock_width) / 2, block_y),
            CLOCK_SCALE,
        );
        draw_digits(
            &mut frame,
            battery_mv,
            BATTERY_DIGITS,
            (
                block_x + (block_width - battery_width) / 2,
                block_y + clock_height + LINE_GAP,
            ),
            BATTERY_SCALE,
        );

        let block =
            self.geometry
                .align_region(Region::new(block_x, block_y, block_width, block_height));
        let mut expanded = vec![0u8; block.rgb565_len()];
        frame.expand_region(block, &mut expanded);

        // The block covers every changed pixel, so writing it fully replaces it.
        self.panel
            .write_region(block, &expanded)
            .map_err(BringUpError::Display)
    }

    /// Puts the panel into sleep, before `power` cuts the enable rail.
    pub fn sleep(&mut self) -> Result<(), BringUpError> {
        self.panel.display_off().map_err(BringUpError::Display)
    }
}
