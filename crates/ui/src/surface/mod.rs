//! Pixel surfaces.
//!
//! Surfaces borrow storage owned by the caller so the UI never allocates and
//! the firmware decides whether a framebuffer lives in SRAM or PSRAM.

mod gray;
mod mono;

pub use gray::GraySurface;
pub use mono::MonoSurface;

use common::Region;

use crate::luma::Luma;

/// A failure to construct a surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceError {
    /// The supplied storage is too small for the requested dimensions.
    BufferTooSmall,
}

/// A writable luminance target.
pub trait Surface {
    /// The surface width in pixels.
    fn width(&self) -> u16;

    /// The surface height in pixels.
    fn height(&self) -> u16;

    /// Fills a region with one luminance value, ignoring out-of-bounds pixels.
    fn fill_rect(&mut self, region: Region, color: Luma);

    /// Sets one pixel, ignoring out-of-bounds coordinates.
    fn set_pixel(&mut self, x: u16, y: u16, color: Luma);

    /// Reads one pixel. Out-of-bounds reads are black.
    fn pixel(&self, x: u16, y: u16) -> Luma;

    /// Clears the whole surface.
    fn clear(&mut self, color: Luma) {
        let region = Region::full(self.width(), self.height());
        self.fill_rect(region, color);
    }
}
