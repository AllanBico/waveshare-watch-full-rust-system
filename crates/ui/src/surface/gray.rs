//! 8-bit grayscale surface.
//!
//! One byte per pixel. Desk mode uses this for tonal surfaces and antialiasing.

use common::Region;

use crate::luma::Luma;
use crate::surface::{Surface, SurfaceError};

/// Grayscale to RGB565 conversion table.
const LUMA_LUT: [u16; 256] = build_luma_lut();

const fn build_luma_lut() -> [u16; 256] {
    let mut table = [0u16; 256];
    let mut value = 0usize;
    while value < 256 {
        let level = value as u16;
        let red = (level >> 3) & 0x1F;
        let green = (level >> 2) & 0x3F;
        let blue = (level >> 3) & 0x1F;
        table[value] = (red << 11) | (green << 5) | blue;
        value += 1;
    }
    table
}

/// An 8-bit grayscale surface borrowing caller-owned storage.
pub struct GraySurface<'a> {
    width: u16,
    height: u16,
    data: &'a mut [u8],
}

impl<'a> GraySurface<'a> {
    /// Wraps storage wide enough for `width` by `height` pixels.
    ///
    /// The storage is cleared on construction.
    pub fn new(width: u16, height: u16, data: &'a mut [u8]) -> Result<Self, SurfaceError> {
        let needed = Self::buffer_len(width, height);
        if data.len() < needed {
            return Err(SurfaceError::BufferTooSmall);
        }
        data[..needed].fill(0);
        Ok(Self {
            width,
            height,
            data,
        })
    }

    /// The storage size needed for these dimensions.
    pub const fn buffer_len(width: u16, height: u16) -> usize {
        (width as usize).saturating_mul(height as usize)
    }

    /// The raw pixel bytes, one per pixel.
    pub fn pixels(&self) -> &[u8] {
        &self.data[..Self::buffer_len(self.width, self.height)]
    }

    /// Expands a region into big-endian RGB565 bytes through the luma LUT.
    ///
    /// Writes `min(out.len(), region.rgb565_len())` bytes and never panics.
    pub fn expand_region(&self, region: Region, out: &mut [u8]) {
        let region = region.intersection(Region::full(self.width, self.height));
        let mut index = 0usize;
        for y in region.y..region.bottom() {
            for x in region.x..region.right() {
                if index + 2 > out.len() {
                    return;
                }
                let source = usize::from(y) * usize::from(self.width) + usize::from(x);
                let value = LUMA_LUT[usize::from(self.data[source])].to_be_bytes();
                out[index] = value[0];
                out[index + 1] = value[1];
                index += 2;
            }
        }
    }
}

impl Surface for GraySurface<'_> {
    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn fill_rect(&mut self, region: Region, color: Luma) {
        let region = region.intersection(Region::full(self.width, self.height));
        for y in region.y..region.bottom() {
            let start = usize::from(y) * usize::from(self.width) + usize::from(region.x);
            let end = start + usize::from(region.width);
            self.data[start..end].fill(color.raw());
        }
    }

    fn set_pixel(&mut self, x: u16, y: u16, color: Luma) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = usize::from(y) * usize::from(self.width) + usize::from(x);
        self.data[index] = color.raw();
    }

    fn pixel(&self, x: u16, y: u16) -> Luma {
        if x >= self.width || y >= self.height {
            return Luma::BLACK;
        }
        let index = usize::from(y) * usize::from(self.width) + usize::from(x);
        Luma::new(self.data[index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: u16 = 4;
    const HEIGHT: u16 = 3;

    fn surface() -> GraySurface<'static> {
        let data = Box::leak(vec![0u8; GraySurface::buffer_len(WIDTH, HEIGHT)].into_boxed_slice());
        GraySurface::new(WIDTH, HEIGHT, data).unwrap()
    }

    #[test]
    fn buffer_len_is_one_byte_per_pixel() {
        assert_eq!(GraySurface::buffer_len(4, 3), 12);
        assert_eq!(GraySurface::buffer_len(410, 502), 205_820);
    }

    #[test]
    fn undersized_storage_is_rejected() {
        let mut data = [0u8; 11];
        assert!(matches!(
            GraySurface::new(WIDTH, HEIGHT, &mut data),
            Err(SurfaceError::BufferTooSmall)
        ));
    }

    #[test]
    fn pixels_store_exact_luminance() {
        let mut surface = surface();
        surface.set_pixel(1, 1, Luma::new(77));
        assert_eq!(surface.pixel(1, 1), Luma::new(77));
        assert_eq!(surface.pixel(0, 0), Luma::BLACK);
    }

    #[test]
    fn fill_clips_to_the_surface() {
        let mut surface = surface();
        surface.fill_rect(Region::new(2, 1, 20, 20), Luma::new(42));
        assert_eq!(surface.pixel(2, 1), Luma::new(42));
        assert_eq!(surface.pixel(3, 2), Luma::new(42));
        assert_eq!(surface.pixel(1, 1), Luma::BLACK);
    }

    #[test]
    fn out_of_bounds_access_is_safe() {
        let mut surface = surface();
        surface.set_pixel(99, 99, Luma::WHITE);
        assert_eq!(surface.pixel(99, 99), Luma::BLACK);
    }
}
