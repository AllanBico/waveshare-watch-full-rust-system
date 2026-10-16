//! CO5300 bring-up on the board (Phase 3.3).
//!
//! Wires the board's SPI2 quad bus to the generic [`Co5300`] driver and shows
//! monochrome partial refresh: render a fixed "10:09" clock into the 1-bit
//! [`MonoBuffer`], expand only the clock region to RGB565, and push it over a
//! black background. The rest of the panel is never transmitted.
//!
//! The transport, confirmed against ChipOne's datasheet and the vendor
//! databus, is one-line `0x02` register access and one-line instruction and
//! address with four-line pixel data for `0x32`.

use alloc::vec;

use esp_hal::delay::Delay;
use esp_hal::dma::{DmaBufError, DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::{
    Address, Command, Config as SpiConfig, ConfigError, DataMode, Spi, SpiDmaBus,
};
use esp_hal::spi::{Error as SpiError, Mode};
use esp_hal::time::Rate;

use common::{Region, Rgb565};
use drivers::board;
use drivers::co5300::{
    Bus, Co5300, DisplayError, Geometry, OP_WRITE_COMMAND, OP_WRITE_PIXELS, PIXEL_WRITE_ADDRESS,
};
use hal::Display;
use ui::{clock_region, clock_size, draw_time, MonoBuffer};

/// SPI clock. The vendor uses 40 MHz.
const SPI_FREQUENCY_MHZ: u32 = 40;

/// DMA staging buffer and pixel chunk, in bytes (even).
const CHUNK_BYTES: usize = 2048;

/// Integer scale for the demo clock.
const CLOCK_SCALE: u16 = 3;

/// Fixed demo time, so the render is deterministic.
const DEMO_HOUR: u8 = 10;
const DEMO_MINUTE: u8 = 9;

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

/// An SPI2 quad transport implementing the driver's [`Bus`].
struct EspQspiBus<'d> {
    bus: SpiDmaBus<'d, esp_hal::Blocking>,
    cs: Output<'d>,
}

impl Bus for EspQspiBus<'_> {
    type Error = SpiError;

    fn write_command(&mut self, command: u8) -> Result<(), Self::Error> {
        self.cs.set_low();
        let result = self.bus.half_duplex_write(
            DataMode::Single,
            Command::_8Bit(OP_WRITE_COMMAND.into(), DataMode::Single),
            Address::_24Bit(u32::from(command) << 8, DataMode::Single),
            0,
            &[],
        );
        self.cs.set_high();
        result
    }

    fn write_command_data(&mut self, command: u8, data: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        let result = self.bus.half_duplex_write(
            DataMode::Single,
            Command::_8Bit(OP_WRITE_COMMAND.into(), DataMode::Single),
            Address::_24Bit(u32::from(command) << 8, DataMode::Single),
            0,
            data,
        );
        self.cs.set_high();
        result
    }

    fn write_pixels(&mut self, pixels: &[u8]) -> Result<(), Self::Error> {
        self.cs.set_low();
        let result = self.stream_pixels(pixels);
        self.cs.set_high();
        result
    }

    fn fill_pixels(&mut self, color: Rgb565, count: u32) -> Result<(), Self::Error> {
        self.cs.set_low();
        let result = self.stream_fill(color, count);
        self.cs.set_high();
        result
    }
}

impl EspQspiBus<'_> {
    /// Sends the `0x32` pixel-write instruction and address prefix.
    fn pixel_prefix(&mut self) -> Result<(), SpiError> {
        self.bus.half_duplex_write(
            DataMode::Single,
            Command::_8Bit(OP_WRITE_PIXELS.into(), DataMode::Single),
            Address::_24Bit(PIXEL_WRITE_ADDRESS, DataMode::Single),
            0,
            &[],
        )
    }

    /// Streams pixel bytes in one chip-select window.
    fn stream_pixels(&mut self, pixels: &[u8]) -> Result<(), SpiError> {
        self.pixel_prefix()?;
        for chunk in pixels.chunks(CHUNK_BYTES) {
            self.bus
                .half_duplex_write(DataMode::Quad, Command::None, Address::None, 0, chunk)?;
        }
        Ok(())
    }

    /// Streams a repeated color in one chip-select window.
    fn stream_fill(&mut self, color: Rgb565, mut count: u32) -> Result<(), SpiError> {
        let bytes = color.to_be_bytes();
        let mut chunk = [0u8; CHUNK_BYTES];
        for pair in chunk.chunks_exact_mut(2) {
            pair.copy_from_slice(&bytes);
        }

        let per_chunk = (CHUNK_BYTES / 2) as u32;
        self.pixel_prefix()?;
        while count > 0 {
            let pixels = count.min(per_chunk) as usize;
            self.bus.half_duplex_write(
                DataMode::Quad,
                Command::None,
                Address::None,
                0,
                &chunk[..pixels * 2],
            )?;
            count -= pixels as u32;
        }
        Ok(())
    }
}

/// Lit-pixel accounting for the rendered demo frame.
#[derive(Debug)]
pub struct DisplayStat {
    /// Number of lit (white) pixels in the frame.
    pub lit_pixels: u32,
    /// Total pixels in the frame.
    pub total_pixels: u32,
}

/// Brings up the panel, draws the demo clock, and flushes only its region.
pub fn run(pins: Peripherals<'static>) -> Result<DisplayStat, BringUpError> {
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
    let mut panel = Co5300::new(bus, reset, Delay::new(), geometry);

    panel.init().map_err(BringUpError::Display)?;

    let mut frame = MonoBuffer::new(geometry.width, geometry.height);
    let (clock_width, clock_height) = clock_size(CLOCK_SCALE);
    let origin = (
        (geometry.width - clock_width) / 2,
        (geometry.height - clock_height) / 2,
    );
    draw_time(&mut frame, DEMO_HOUR, DEMO_MINUTE, origin, CLOCK_SCALE);

    let clock = geometry.align_region(clock_region(origin, CLOCK_SCALE));
    let mut expanded = vec![0u8; clock.rgb565_len()];
    frame.expand_region(clock, &mut expanded);

    panel
        .fill_region(Region::full(geometry.width, geometry.height), Rgb565::BLACK)
        .map_err(BringUpError::Display)?;
    panel
        .write_region(clock, &expanded)
        .map_err(BringUpError::Display)?;

    Ok(DisplayStat {
        lit_pixels: frame.lit_pixels(),
        total_pixels: frame.total_pixels(),
    })
}
