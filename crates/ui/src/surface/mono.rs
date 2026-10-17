//! 1-bit monochrome surface.
//!
//! One bit per pixel, MSB first within each row, rows padded to a byte. Lit
//! pixels are white and unlit pixels are black.

use common::{Region, Rgb565};

use crate::luma::Luma;
use crate::surface::{Surface, SurfaceError};

const LIT: [u8; 2] = Rgb565::WHITE.to_be_bytes();
const UNLIT: [u8; 2] = Rgb565::BLACK.to_be_bytes();

/// A 1-bit surface borrowing caller-owned storage.
pub struct MonoSurface<'a> {
    width: u16,
    height: u16,
    stride: usize,
    data: &'a mut [u8],
}

impl<'a> MonoSurface<'a> {
    /// Wraps storage wide enough for `width` by `height` pixels.
    ///
    /// The storage is cleared on construction.
    pub fn new(width: u16, height: u16, data: &'a mut [u8]) -> Result<Self, SurfaceError> {
        let stride = stride_bytes(width);
        let needed = stride.saturating_mul(usize::from(height));
        if data.len() < needed {
            return Err(SurfaceError::BufferTooSmall);
        }
        data[..needed].fill(0);
        Ok(Self {
            width,
            height,
            stride,
            data,
        })
    }

    /// The storage size needed for these dimensions.
    pub const fn buffer_len(width: u16, height: u16) -> usize {
        stride_bytes(width).saturating_mul(height as usize)
    }

    /// The number of lit pixels.
    pub fn lit_pixels(&self) -> u32 {
        self.data[..self.stride * usize::from(self.height)]
            .iter()
            .map(|byte| byte.count_ones())
            .sum()
    }

    /// Expands a region into big-endian RGB565 bytes.
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
                let color = if self.is_lit(x, y) { LIT } else { UNLIT };
                out[index] = color[0];
                out[index + 1] = color[1];
                index += 2;
            }
        }
    }

    fn is_lit(&self, x: u16, y: u16) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let index = usize::from(y) * self.stride + (usize::from(x) >> 3);
        self.data[index] & (0b1000_0000 >> (x & 7)) != 0
    }
}

impl Surface for MonoSurface<'_> {
    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn fill_rect(&mut self, region: Region, color: Luma) {
        let region = region.intersection(Region::full(self.width, self.height));
        let lit = color.is_lit();
        for y in region.y..region.bottom() {
            let row = usize::from(y) * self.stride;
            for x in region.x..region.right() {
                let index = row + (usize::from(x) >> 3);
                let mask = 0b1000_0000 >> (x & 7);
                if lit {
                    self.data[index] |= mask;
                } else {
                    self.data[index] &= !mask;
                }
            }
        }
    }

    fn set_pixel(&mut self, x: u16, y: u16, color: Luma) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = usize::from(y) * self.stride + (usize::from(x) >> 3);
        let mask = 0b1000_0000 >> (x & 7);
        if color.is_lit() {
            self.data[index] |= mask;
        } else {
            self.data[index] &= !mask;
        }
    }

    fn pixel(&self, x: u16, y: u16) -> Luma {
        if self.is_lit(x, y) {
            Luma::WHITE
        } else {
            Luma::BLACK
        }
    }
}

/// Bytes per row for a `width`-pixel 1-bit row.
pub const fn stride_bytes(width: u16) -> usize {
    (width as usize).div_ceil(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    const WIDTH: u16 = 16;
    const HEIGHT: u16 = 4;

    fn surface() -> MonoSurface<'static> {
        let data = Box::leak(vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)].into_boxed_slice());
        MonoSurface::new(WIDTH, HEIGHT, data).unwrap()
    }

    #[test]
    fn buffer_len_rounds_rows_to_a_byte() {
        assert_eq!(MonoSurface::buffer_len(1, 1), 1);
        assert_eq!(MonoSurface::buffer_len(9, 2), 4);
        assert_eq!(MonoSurface::buffer_len(410, 502), 52 * 502);
    }

    #[test]
    fn undersized_storage_is_rejected() {
        let mut data = [0u8; 1];
        assert!(matches!(
            MonoSurface::new(9, 1, &mut data),
            Err(SurfaceError::BufferTooSmall)
        ));
    }

    #[test]
    fn set_and_read_round_trip() {
        let mut surface = surface();
        surface.set_pixel(0, 0, Luma::WHITE);
        surface.set_pixel(15, 3, Luma::WHITE);
        assert_eq!(surface.pixel(0, 0), Luma::WHITE);
        assert_eq!(surface.pixel(15, 3), Luma::WHITE);
        assert_eq!(surface.pixel(1, 0), Luma::BLACK);
        assert_eq!(surface.lit_pixels(), 2);
    }

    #[test]
    fn out_of_bounds_access_is_safe() {
        let mut surface = surface();
        surface.set_pixel(99, 99, Luma::WHITE);
        assert_eq!(surface.pixel(99, 99), Luma::BLACK);
        assert_eq!(surface.lit_pixels(), 0);
    }

    #[test]
    fn fill_clips_to_the_surface() {
        let mut surface = surface();
        surface.fill_rect(Region::new(8, 2, 20, 20), Luma::WHITE);
        assert_eq!(surface.lit_pixels(), 8 * 2);
    }

    #[test]
    fn expansion_writes_big_endian_rgb565() {
        let mut surface = surface();
        surface.set_pixel(1, 0, Luma::WHITE);
        let mut out = [0u8; 8];
        surface.expand_region(Region::new(0, 0, 4, 1), &mut out);
        assert_eq!(out, [0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00]);
    }
}
