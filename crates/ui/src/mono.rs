//! 1-bit monochrome framebuffer.
//!
//! One bit per pixel, MSB-first within each row, rows padded to a byte
//! boundary. This is the watch-mode render target: black pixels are off, so
//! only the lit pixels get pushed to the panel.

use alloc::vec;
use alloc::vec::Vec;

use common::{Region, Rgb565};

/// Wire bytes for a lit pixel (white) and an unlit pixel (off black).
const LIT: [u8; 2] = Rgb565::WHITE.to_be_bytes();
const UNLIT: [u8; 2] = Rgb565::BLACK.to_be_bytes();

/// A 1-bit framebuffer with a byte-aligned stride.
pub struct MonoBuffer {
    width: u16,
    height: u16,
    stride: usize,
    data: Vec<u8>,
}

impl MonoBuffer {
    /// Allocates a cleared buffer of `width` by `height` pixels.
    pub fn new(width: u16, height: u16) -> Self {
        let stride = stride_bytes(width);
        Self {
            width,
            height,
            stride,
            data: vec![0u8; stride * height as usize],
        }
    }

    /// The buffer width in pixels.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// The buffer height in pixels.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// The total number of pixels.
    pub fn total_pixels(&self) -> u32 {
        u32::from(self.width) * u32::from(self.height)
    }

    /// Clears every pixel to unlit.
    pub fn clear(&mut self) {
        self.data.fill(0);
    }

    /// Sets a pixel, ignoring coordinates outside the buffer.
    pub fn set(&mut self, x: u16, y: u16, lit: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        let index = y as usize * self.stride + (x as usize >> 3);
        let mask = 0b1000_0000 >> (x & 7);
        if lit {
            self.data[index] |= mask;
        } else {
            self.data[index] &= !mask;
        }
    }

    /// Whether a pixel is lit. Out-of-bounds reads are unlit.
    pub fn is_lit(&self, x: u16, y: u16) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let index = y as usize * self.stride + (x as usize >> 3);
        self.data[index] & (0b1000_0000 >> (x & 7)) != 0
    }

    /// Sets or clears every pixel in a region.
    pub fn fill_rect(&mut self, region: Region, lit: bool) {
        for y in region.y..region.y.saturating_add(region.height) {
            for x in region.x..region.x.saturating_add(region.width) {
                self.set(x, y, lit);
            }
        }
    }

    /// The number of lit pixels.
    pub fn lit_pixels(&self) -> u32 {
        self.data.iter().map(|byte| byte.count_ones()).sum()
    }

    /// Expands a region into big-endian RGB565 bytes.
    ///
    /// Writes `min(out.len(), region.rgb565_len())` bytes and never panics.
    pub fn expand_region(&self, region: Region, out: &mut [u8]) {
        let mut index = 0usize;
        for y in region.y..region.y.saturating_add(region.height) {
            for x in region.x..region.x.saturating_add(region.width) {
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
}

/// Bytes per row for a `width`-pixel 1-bit row.
pub const fn stride_bytes(width: u16) -> usize {
    (width as usize).div_ceil(8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stride_rounds_up_to_a_byte() {
        assert_eq!(stride_bytes(0), 0);
        assert_eq!(stride_bytes(1), 1);
        assert_eq!(stride_bytes(8), 1);
        assert_eq!(stride_bytes(9), 2);
        assert_eq!(stride_bytes(410), 52);
    }

    #[test]
    fn set_and_is_lit_round_trip() {
        let mut buffer = MonoBuffer::new(16, 2);
        buffer.set(0, 0, true);
        buffer.set(15, 1, true);
        assert!(buffer.is_lit(0, 0));
        assert!(buffer.is_lit(15, 1));
        assert!(!buffer.is_lit(1, 0));
        assert_eq!(buffer.lit_pixels(), 2);

        buffer.set(0, 0, false);
        assert_eq!(buffer.lit_pixels(), 1);
    }

    #[test]
    fn out_of_bounds_access_is_safe() {
        let mut buffer = MonoBuffer::new(4, 4);
        buffer.set(100, 100, true);
        assert!(!buffer.is_lit(100, 100));
        assert_eq!(buffer.lit_pixels(), 0);
    }

    #[test]
    fn clear_resets_every_pixel() {
        let mut buffer = MonoBuffer::new(10, 10);
        buffer.fill_rect(Region::full(10, 10), true);
        assert_eq!(buffer.lit_pixels(), 100);
        buffer.clear();
        assert_eq!(buffer.lit_pixels(), 0);
    }

    #[test]
    fn expand_region_writes_white_and_black() {
        let mut buffer = MonoBuffer::new(4, 1);
        buffer.set(1, 0, true);
        let region = Region::new(0, 0, 4, 1);
        let mut out = [0u8; 8];
        buffer.expand_region(region, &mut out);
        assert_eq!(out, [0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00]);
    }

    #[test]
    fn expand_region_never_overruns_the_output() {
        let mut buffer = MonoBuffer::new(8, 2);
        buffer.fill_rect(Region::full(8, 2), true);
        let mut out = [0u8; 4];
        buffer.expand_region(Region::new(0, 0, 8, 2), &mut out);
        assert_eq!(out, [0xFF, 0xFF, 0xFF, 0xFF]);
    }
}
