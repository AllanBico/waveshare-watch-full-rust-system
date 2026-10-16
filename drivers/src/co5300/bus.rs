//! CO5300 quad-SPI transport contract.
//!
//! The bus owns chip-select framing and DMA; the driver owns the command set
//! and timing. An implementation must hold chip select low for the whole of a
//! pixel transfer: the panel treats the stream as one continuous RAM write.

use common::Rgb565;

/// A quad-SPI transport to a CO5300 panel.
pub trait Bus {
    /// Transport error.
    type Error;

    /// Sends a parameter-less driver command.
    fn write_command(&mut self, command: u8) -> Result<(), Self::Error>;

    /// Sends a driver command followed by its parameters.
    fn write_command_data(&mut self, command: u8, data: &[u8]) -> Result<(), Self::Error>;

    /// Streams big-endian RGB565 pixels into the active window.
    fn write_pixels(&mut self, pixels: &[u8]) -> Result<(), Self::Error>;

    /// Fills the active window with `count` pixels of one color.
    fn fill_pixels(&mut self, color: Rgb565, count: u32) -> Result<(), Self::Error>;
}
