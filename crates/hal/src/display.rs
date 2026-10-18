//! Display contract.
//!
//! The panel-level interface a driver fulfills and the renderer consumes:
//! geometry, power/brightness, and region writes. Color interpretation lives
//! above this trait; the byte format here is big-endian RGB565.

use common::{Region, Rgb565};

/// A writable pixel display.
pub trait Display {
    /// Driver-specific error.
    type Error;

    /// The panel width in pixels.
    fn width(&self) -> u16;

    /// The panel height in pixels.
    fn height(&self) -> u16;

    /// Turns the panel on (exits sleep).
    fn display_on(&mut self) -> Result<(), Self::Error>;

    /// Turns the panel off (enters sleep).
    fn display_off(&mut self) -> Result<(), Self::Error>;

    /// Sets the panel brightness, `0x00` to `0xFF`.
    fn set_brightness(&mut self, level: u8) -> Result<(), Self::Error>;

    /// Fills a region with a single color.
    fn fill_region(&mut self, region: Region, color: Rgb565) -> Result<(), Self::Error>;

    /// Writes big-endian RGB565 pixels into a region.
    ///
    /// `pixels.len()` must equal `region.rgb565_len()`.
    fn write_region(&mut self, region: Region, pixels: &[u8]) -> Result<(), Self::Error>;

    /// Blocks until the next tearing-effect edge.
    ///
    /// Targets without a TE signal may keep the default no-op.
    fn wait_for_vsync(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
