//! Host-only golden-frame helpers.

use alloc::vec::Vec;

use crate::luma::Luma;
use crate::surface::{GraySurface, MonoSurface, Surface};

/// FNV-1a over a byte slice.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// A binary PPM (P6) rendering of a monochrome surface.
pub fn mono_ppm(surface: &MonoSurface<'_>) -> Vec<u8> {
    let mut out = header(surface.width(), surface.height());
    for y in 0..surface.height() {
        for x in 0..surface.width() {
            let value = if surface.pixel(x, y) == Luma::WHITE {
                255
            } else {
                0
            };
            out.extend_from_slice(&[value, value, value]);
        }
    }
    out
}

/// A binary PPM (P6) rendering of a grayscale surface.
pub fn gray_ppm(surface: &GraySurface<'_>) -> Vec<u8> {
    let mut out = header(surface.width(), surface.height());
    for y in 0..surface.height() {
        for x in 0..surface.width() {
            let value = surface.pixel(x, y).raw();
            out.extend_from_slice(&[value, value, value]);
        }
    }
    out
}

fn header(width: u16, height: u16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"P6\n");
    out.extend_from_slice(format!("{width} {height}\n255\n").as_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::Surface;

    #[test]
    fn fnv1a_matches_the_known_vector() {
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
    }

    #[test]
    fn mono_ppm_has_a_header_and_three_bytes_per_pixel() {
        let mut data = vec![0u8; MonoSurface::buffer_len(2, 1)];
        let mut surface = MonoSurface::new(2, 1, &mut data).unwrap();
        surface.set_pixel(0, 0, Luma::WHITE);
        let ppm = mono_ppm(&surface);
        assert!(ppm.starts_with(b"P6\n2 1\n255\n"));
        assert_eq!(ppm.len(), 11 + 6);
    }

    #[test]
    fn gray_ppm_has_a_header_and_three_bytes_per_pixel() {
        let mut data = vec![0u8; GraySurface::buffer_len(2, 1)];
        let mut surface = GraySurface::new(2, 1, &mut data).unwrap();
        surface.set_pixel(1, 0, Luma::new(9));
        let ppm = gray_ppm(&surface);
        assert!(ppm.starts_with(b"P6\n2 1\n255\n"));
        assert_eq!(ppm.len(), 11 + 6);
    }
}
