//! CO5300 QSPI bring-up spike (Phase 3.0).
//!
//! Proves that SPI2 half-duplex quad writes reach the CO5300 AMOLED: hardware
//! reset, the vendor init table, and a full-screen solid fill. There is no
//! framebuffer and no dirty-rect support; the real driver lands in Phase 3.1.
//!
//! Transport, confirmed against ChipOne's datasheet and the vendor databus:
//! register access uses the `0x02` instruction on one line with a 24-bit
//! address `{8'h00, command, 8'h00}`; pixel data uses the `0x32` instruction on
//! one line for instruction and address, then four-line data.

use esp_hal::delay::Delay;
use esp_hal::dma::{DmaBufError, DmaRxBuf, DmaTxBuf};
use esp_hal::dma_buffers;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::spi::master::{
    Address, Command, Config as SpiConfig, ConfigError, DataMode, Spi, SpiDmaBus,
};
use esp_hal::spi::{Error as SpiError, Mode};
use esp_hal::time::Rate;

use drivers::board;

/// QSPI instruction: command / register access.
const OP_WRITE_COMMAND: u8 = 0x02;
/// QSPI instruction: four-line pixel write with a 24-bit address.
const OP_WRITE_PIXELS: u8 = 0x32;
/// RAMWR-continuous command encoded as the pixel-write address.
const PIXEL_WRITE_ADDRESS: u32 = 0x0000_3C00;

// CO5300 driver-IC commands used during bring-up.
const CMD_SLPOUT: u8 = 0x11;
const CMD_INVOFF: u8 = 0x20;
const CMD_DISPON: u8 = 0x29;
const CMD_CASET: u8 = 0x2A;
const CMD_PASET: u8 = 0x2B;
const CMD_RAMWR: u8 = 0x2C;
const CMD_MADCTL: u8 = 0x36;
const CMD_PIXFMT: u8 = 0x3A;
const CMD_BRIGHTNESS: u8 = 0x51;
const CMD_WCTRLD1: u8 = 0x53;
const CMD_WCE: u8 = 0x58;
const CMD_BRIGHTNESS_HBM: u8 = 0x63;
const CMD_SPIMODECTL: u8 = 0xC4;
const CMD_WRITE_CMD_PAGE: u8 = 0xFE;

/// SPI clock. The vendor uses 40 MHz.
const SPI_FREQUENCY_MHZ: u32 = 40;

// Reset and init timing from the vendor table.
const RESET_PULSE_MS: u32 = 10;
const RESET_HOLD_MS: u32 = 200;
const SLPOUT_MS: u32 = 120;
const INIT_SETTLE_MS: u32 = 10;

/// DMA staging buffer for pixel data, in bytes (even).
const PIXEL_BUFFER_BYTES: usize = 2048;

/// Placeholder fill colour for the spike: RGB565 red.
const SPIKE_COLOR: u16 = 0xF800;

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
pub enum DisplayError {
    /// SPI bus configuration failed.
    Config(ConfigError),
    /// DMA buffer construction failed.
    Dma(DmaBufError),
    /// An SPI transfer failed.
    Spi(SpiError),
}

/// A CO5300 panel on an SPI2 quad bus.
struct Panel<'d> {
    bus: SpiDmaBus<'d, esp_hal::Blocking>,
    cs: Output<'d>,
    reset: Output<'d>,
    delay: Delay,
}

/// Brings up the panel and paints a solid fill. Phase 3.0 hardware check.
pub fn run(pins: Peripherals<'static>) -> Result<(), DisplayError> {
    let config = SpiConfig::default()
        .with_frequency(Rate::from_mhz(SPI_FREQUENCY_MHZ))
        .with_mode(Mode::_0);

    let (rx_buffer, rx_descriptors, tx_buffer, tx_descriptors) =
        dma_buffers!(4, PIXEL_BUFFER_BYTES);
    let rx = DmaRxBuf::new(rx_descriptors, rx_buffer).map_err(DisplayError::Dma)?;
    let tx = DmaTxBuf::new(tx_descriptors, tx_buffer).map_err(DisplayError::Dma)?;

    let bus = Spi::new(pins.spi, config)
        .map_err(DisplayError::Config)?
        .with_sck(pins.sclk)
        .with_sio0(pins.sio0)
        .with_sio1(pins.sio1)
        .with_sio2(pins.sio2)
        .with_sio3(pins.sio3)
        .with_dma(pins.dma)
        .with_buffers(rx, tx);

    let mut panel = Panel {
        bus,
        cs: Output::new(pins.cs, Level::High, OutputConfig::default()),
        reset: Output::new(pins.reset, Level::High, OutputConfig::default()),
        delay: Delay::new(),
    };

    panel.init()?;
    panel.fill_screen(SPIKE_COLOR)
}

impl Panel<'_> {
    /// Runs the vendor init table. Geometry is not set here.
    fn init(&mut self) -> Result<(), DisplayError> {
        self.reset.set_high();
        self.delay.delay_millis(RESET_PULSE_MS);
        self.reset.set_low();
        self.delay.delay_millis(RESET_HOLD_MS);
        self.reset.set_high();
        self.delay.delay_millis(RESET_HOLD_MS);

        self.write_command(CMD_SLPOUT)?;
        self.delay.delay_millis(SLPOUT_MS);

        self.write_command_data(CMD_WRITE_CMD_PAGE, &[0x00])?;
        self.write_command_data(CMD_SPIMODECTL, &[0x80])?;
        self.write_command_data(CMD_PIXFMT, &[0x55])?;
        self.write_command_data(CMD_WCTRLD1, &[0x20])?;
        self.write_command_data(CMD_BRIGHTNESS_HBM, &[0xFF])?;
        self.write_command(CMD_DISPON)?;
        self.write_command_data(CMD_BRIGHTNESS, &[0xD0])?;
        self.write_command_data(CMD_WCE, &[0x00])?;
        self.delay.delay_millis(INIT_SETTLE_MS);

        self.write_command(CMD_INVOFF)?;
        self.write_command_data(CMD_MADCTL, &[0x00])
    }

    /// Fills the whole panel with one RGB565 colour.
    fn fill_screen(&mut self, color: u16) -> Result<(), DisplayError> {
        self.set_window(0, 0, board::LCD_WIDTH, board::LCD_HEIGHT)?;

        let bytes = color.to_be_bytes();
        let mut chunk = [0u8; PIXEL_BUFFER_BYTES];
        for pair in chunk.chunks_exact_mut(2) {
            pair.copy_from_slice(&bytes);
        }

        self.cs.set_low();
        let result = self.stream_fill(&chunk);
        self.cs.set_high();
        result
    }

    /// Streams the fill pattern while chip select is held low.
    fn stream_fill(&mut self, chunk: &[u8]) -> Result<(), DisplayError> {
        self.write_pixels_command()?;
        let per_chunk = (chunk.len() / 2) as u32;
        let mut remaining = u32::from(board::LCD_WIDTH) * u32::from(board::LCD_HEIGHT);
        while remaining > 0 {
            let pixels = remaining.min(per_chunk) as usize;
            self.write_pixels(&chunk[..pixels * 2])?;
            remaining -= pixels as u32;
        }
        Ok(())
    }

    /// Sets the RAM address window and starts a memory write.
    fn set_window(&mut self, x: u16, y: u16, w: u16, h: u16) -> Result<(), DisplayError> {
        let x0 = x + board::LCD_COL_OFFSET;
        let y0 = y + board::LCD_ROW_OFFSET;
        let x1 = x0 + w - 1;
        let y1 = y0 + h - 1;
        self.write_command_data(CMD_CASET, &[high(x0), low(x0), high(x1), low(x1)])?;
        self.write_command_data(CMD_PASET, &[high(y0), low(y0), high(y1), low(y1)])?;
        self.write_command(CMD_RAMWR)
    }

    /// Sends a parameter-less driver command.
    fn write_command(&mut self, command: u8) -> Result<(), DisplayError> {
        self.cs.set_low();
        let result = self.write_command_transfer(command, &[]);
        self.cs.set_high();
        result
    }

    /// Sends a driver command followed by its parameters.
    fn write_command_data(&mut self, command: u8, data: &[u8]) -> Result<(), DisplayError> {
        self.cs.set_low();
        let result = self.write_command_transfer(command, data);
        self.cs.set_high();
        result
    }

    /// The `0x02` instruction transfer shared by both command forms.
    fn write_command_transfer(&mut self, command: u8, data: &[u8]) -> Result<(), DisplayError> {
        self.bus
            .half_duplex_write(
                DataMode::Single,
                Command::_8Bit(OP_WRITE_COMMAND.into(), DataMode::Single),
                Address::_24Bit(u32::from(command) << 8, DataMode::Single),
                0,
                data,
            )
            .map_err(DisplayError::Spi)
    }

    /// Sends the `0x32` pixel-write instruction and address prefix.
    fn write_pixels_command(&mut self) -> Result<(), DisplayError> {
        self.bus
            .half_duplex_write(
                DataMode::Single,
                Command::_8Bit(OP_WRITE_PIXELS.into(), DataMode::Single),
                Address::_24Bit(PIXEL_WRITE_ADDRESS, DataMode::Single),
                0,
                &[],
            )
            .map_err(DisplayError::Spi)
    }

    /// Streams four-line pixel data. Chip select must already be low.
    fn write_pixels(&mut self, bytes: &[u8]) -> Result<(), DisplayError> {
        self.bus
            .half_duplex_write(DataMode::Quad, Command::None, Address::None, 0, bytes)
            .map_err(DisplayError::Spi)
    }
}

fn high(value: u16) -> u8 {
    (value >> 8) as u8
}

fn low(value: u16) -> u8 {
    value as u8
}
