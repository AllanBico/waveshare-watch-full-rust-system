//! SPI2 quad transport for the CO5300.
//!
//! Implements the driver's [`Bus`] on the board's SPI2 peripheral: register
//! access is one-line `0x02`, pixel data is one-line instruction and address
//! with four-line data for `0x32`. Chip select is driven manually so it can
//! stay low across a chunked pixel stream.

use esp_hal::gpio::Output;
use esp_hal::spi::master::{Address, Command, DataMode, SpiDmaBus};
use esp_hal::spi::Error as SpiError;

use common::Rgb565;
use drivers::co5300::{Bus, OP_WRITE_COMMAND, OP_WRITE_PIXELS, PIXEL_WRITE_ADDRESS};

use super::CHUNK_BYTES;

/// An SPI2 quad transport implementing the driver's [`Bus`].
pub(super) struct EspQspiBus<'d> {
    pub(super) bus: SpiDmaBus<'d, esp_hal::Blocking>,
    pub(super) cs: Output<'d>,
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
