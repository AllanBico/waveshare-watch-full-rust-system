use super::*;
use crate::surface::{GraySurface, MonoSurface};

const WIDTH: u16 = 9;
const HEIGHT: u16 = 9;

fn with_painter(run: impl FnOnce(&mut Painter<'_>)) {
    let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    let mut painter = Painter::new(&mut surface);
    run(&mut painter);
}

#[test]
fn root_clip_is_the_full_surface() {
    with_painter(|painter| {
        assert_eq!(painter.clip(), Region::full(WIDTH, HEIGHT));
        assert!(!painter.pop_clip());
    });
}

#[test]
fn clip_stops_drawing_outside() {
    let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    {
        let mut painter = Painter::new(&mut surface);
        assert!(painter.push_clip(Region::new(2, 2, 2, 2)));
        painter.clear(Luma::WHITE);
        assert_eq!(painter.clip(), Region::new(2, 2, 2, 2));
        assert!(painter.pop_clip());
    }
    assert_eq!(surface.lit_pixels(), 4);
    assert_eq!(surface.pixel(2, 2), Luma::WHITE);
    assert_eq!(surface.pixel(1, 1), Luma::BLACK);
}

#[test]
fn stroke_rect_draws_the_perimeter() {
    let mut data = vec![0u8; MonoSurface::buffer_len(5, 5)];
    let mut surface = MonoSurface::new(5, 5, &mut data).unwrap();
    let mut painter = Painter::new(&mut surface);
    painter.stroke_rect(Region::new(0, 0, 5, 5), 1, Luma::WHITE);
    assert_eq!(surface.lit_pixels(), 16);
    assert_eq!(surface.pixel(2, 2), Luma::BLACK);
}

#[test]
fn rounded_rect_leaves_corners_empty() {
    let mut data = vec![0u8; MonoSurface::buffer_len(9, 9)];
    let mut surface = MonoSurface::new(9, 9, &mut data).unwrap();
    let mut painter = Painter::new(&mut surface);
    painter.fill_rounded_rect(Region::new(0, 0, 9, 9), 4, Luma::WHITE);
    assert_eq!(surface.pixel(0, 0), Luma::BLACK);
    assert_eq!(surface.pixel(8, 0), Luma::BLACK);
    assert_eq!(surface.pixel(4, 4), Luma::WHITE);
}

#[test]
fn blend_pixel_mixes_coverage() {
    let mut data = vec![0u8; GraySurface::buffer_len(1, 1)];
    let mut surface = GraySurface::new(1, 1, &mut data).unwrap();
    {
        let mut painter = Painter::new(&mut surface);
        painter.blend_pixel(0, 0, Luma::WHITE, 15);
    }
    assert_eq!(surface.pixel(0, 0), Luma::WHITE);
    {
        let mut painter = Painter::new(&mut surface);
        painter.blend_pixel(0, 0, Luma::BLACK, 7);
    }
    assert_eq!(surface.pixel(0, 0), Luma::new(136));
}

#[test]
fn line_draws_every_step() {
    let mut data = vec![0u8; MonoSurface::buffer_len(5, 5)];
    let mut surface = MonoSurface::new(5, 5, &mut data).unwrap();
    let mut painter = Painter::new(&mut surface);
    painter.line(0, 0, 4, 4, Luma::WHITE);
    assert_eq!(surface.lit_pixels(), 5);
    assert_eq!(surface.pixel(2, 2), Luma::WHITE);
}

#[test]
fn shape_clip_trims_the_corners() {
    let mut data = vec![0u8; MonoSurface::buffer_len(10, 10)];
    let mut surface = MonoSurface::new(10, 10, &mut data).unwrap();
    let mut painter = Painter::new(&mut surface);
    painter.set_shape(Some(ScreenShape {
        corner_radius: 5,
        content_inset: 0,
        crown_inset: 0,
    }));
    painter.fill_rect(Region::full(10, 10), Luma::WHITE);
    assert_eq!(surface.pixel(0, 0), Luma::BLACK);
    assert_eq!(surface.pixel(9, 9), Luma::BLACK);
    assert_eq!(surface.pixel(5, 1), Luma::WHITE);
    assert_eq!(surface.pixel(5, 5), Luma::WHITE);
}
