//! Paint and flush tests.

use common::Region;
use hal::Display;

use super::*;
use crate::luma::Luma;
use crate::scene::Damage;
use crate::surface::{GraySurface, MonoSurface, Surface};

#[derive(Default)]
struct MockDisplay {
    regions: Vec<Region>,
    writes: Vec<Vec<u8>>,
    vsyncs: usize,
}

impl Display for MockDisplay {
    type Error = ();

    fn width(&self) -> u16 {
        16
    }

    fn height(&self) -> u16 {
        16
    }

    fn display_on(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn display_off(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn set_brightness(&mut self, _level: u8) -> Result<(), Self::Error> {
        Ok(())
    }

    fn fill_region(&mut self, _region: Region, _color: common::Rgb565) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write_region(&mut self, region: Region, pixels: &[u8]) -> Result<(), Self::Error> {
        self.regions.push(region);
        self.writes.push(pixels.to_vec());
        Ok(())
    }

    fn wait_for_vsync(&mut self) -> Result<(), Self::Error> {
        self.vsyncs += 1;
        Ok(())
    }
}

#[test]
fn mono_flush_aligns_and_splits_into_strips() {
    let mut data = vec![0u8; MonoSurface::buffer_len(8, 4)];
    let mut surface = MonoSurface::new(8, 4, &mut data).unwrap();
    surface.set_pixel(1, 1, Luma::WHITE);
    let mut damage = Damage::new();
    damage.add(Region::new(1, 1, 3, 3));
    let mut staging = [0u8; 16];
    let mut display = MockDisplay::default();
    flush(&mut display, &surface, &damage, &mut staging).unwrap();
    assert_eq!(display.vsyncs, 1);
    assert_eq!(
        display.regions,
        vec![Region::new(0, 0, 4, 2), Region::new(0, 2, 4, 2)]
    );
    assert_eq!(display.writes[0][10..12], [0xFF, 0xFF]);
}

#[test]
fn gray_flush_uses_the_luma_lut() {
    let mut data = vec![0u8; GraySurface::buffer_len(2, 2)];
    let mut surface = GraySurface::new(2, 2, &mut data).unwrap();
    surface.set_pixel(0, 0, Luma::new(0));
    surface.set_pixel(1, 0, Luma::new(255));
    surface.set_pixel(0, 1, Luma::new(128));
    let damage = Damage::full(Region::new(0, 0, 2, 2));
    let mut staging = [0u8; 8];
    let mut display = MockDisplay::default();
    flush(&mut display, &surface, &damage, &mut staging).unwrap();
    assert_eq!(display.writes[0][..6], [0x00, 0x00, 0xFF, 0xFF, 0x84, 0x10]);
}

#[test]
fn flush_rejects_a_staging_buffer_that_cannot_hold_two_rows() {
    let mut data = vec![0u8; MonoSurface::buffer_len(8, 4)];
    let surface = MonoSurface::new(8, 4, &mut data).unwrap();
    let damage = Damage::full(Region::new(0, 0, 8, 4));
    let mut staging = [0u8; 7];
    let mut display = MockDisplay::default();
    assert_eq!(
        flush(&mut display, &surface, &damage, &mut staging),
        Err(FlushError::StagingTooSmall)
    );
}

#[test]
fn paint_repairs_only_the_damaged_region() {
    let mut data = vec![0u8; GraySurface::buffer_len(16, 16)];
    let mut surface = GraySurface::new(16, 16, &mut data).unwrap();
    let mut damage = Damage::new();
    damage.add(Region::new(4, 4, 4, 4));
    paint_damage(&mut surface, &damage, Luma::new(10), |_, _| {});
    assert_eq!(surface.pixel(4, 4), Luma::new(10));
    assert_eq!(surface.pixel(0, 0), Luma::BLACK);
}

#[test]
fn paint_clips_the_callback_to_each_dirty_region() {
    let mut data = vec![0u8; GraySurface::buffer_len(16, 16)];
    let mut surface = GraySurface::new(16, 16, &mut data).unwrap();
    let mut damage = Damage::new();
    damage.add(Region::new(4, 4, 4, 4));
    paint_damage(&mut surface, &damage, Luma::BLACK, |painter, _| {
        painter.fill_rect(Region::full(16, 16), Luma::WHITE);
    });
    assert_eq!(surface.pixel(4, 4), Luma::WHITE);
    assert_eq!(surface.pixel(3, 4), Luma::BLACK);
    assert_eq!(surface.pixel(8, 4), Luma::BLACK);
}
