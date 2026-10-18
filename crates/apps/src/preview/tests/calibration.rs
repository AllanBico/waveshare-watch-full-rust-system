//! Calibration screen tests.

use common::Mode;
use ui::{GraySurface, MonoSurface, Theme};

use super::*;

#[test]
fn calibration_draws_the_candidate_outline_on_both_surfaces() {
    let mut mono = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut watch = watch_surface(&mut mono);
    let mut watch_preview = preview(Mode::Watch);
    watch_preview.screen = Screen::Calibrate;
    let damage = watch_preview
        .render(&mut watch, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert!(damage.is_full());
    assert!(watch.pixel(0, VIEWPORT.height / 2).is_lit());
    assert!(!watch.pixel(0, 0).is_lit());

    let mut gray = vec![0u8; GraySurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut desk = GraySurface::new(VIEWPORT.width, VIEWPORT.height, &mut gray).unwrap();
    let mut desk_preview = preview(Mode::Desk);
    desk_preview.screen = Screen::Calibrate;
    desk_preview
        .render(&mut desk, &Theme::DESK, VIEWPORT)
        .unwrap();
    assert!(gray.iter().any(|&pixel| pixel != 0));
}

#[test]
fn calibration_controls_adjust_and_reset_the_shape() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview.screen = Screen::Calibrate;
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let initial_radius = preview.calibrate.radii()[0];
    let initial_margin = preview.calibrate.margin();

    let plus = hit(&preview, calibrate::RADIUS_PLUS_1_ID);
    assert!(preview.tap(plus.rect.x + 1, plus.rect.y + 1, VIEWPORT));
    assert_eq!(preview.calibrate.radii()[0], initial_radius + 1);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();

    let corner = hit(&preview, calibrate::CORNER_ID);
    assert!(preview.tap(corner.rect.x + 1, corner.rect.y + 1, VIEWPORT));
    assert_eq!(preview.calibrate.corner(), Some(0));
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();

    let clip = hit(&preview, calibrate::CLIP_ID);
    assert!(preview.tap(clip.rect.x + 1, clip.rect.y + 1, VIEWPORT));
    assert!(preview.calibrate.clip_ui());
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();

    let reset = hit(&preview, calibrate::RESET_ID);
    assert!(preview.tap(reset.rect.x + 1, reset.rect.y + 1, VIEWPORT));
    assert_eq!(preview.calibrate.radii()[0], initial_radius);
    assert_eq!(preview.calibrate.margin(), initial_margin);
    assert_eq!(preview.calibrate.corner(), None);
    assert!(!preview.calibrate.clip_ui());

    assert!(preview.back());
    assert_eq!(preview.screen(), Screen::Home);
}

#[test]
fn the_drawer_opens_the_calibration_screen() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    preview.open_drawer();
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let calibrate = hit(&preview, drawer::CALIBRATE_ID);
    assert!(preview.tap(calibrate.rect.x + 1, calibrate.rect.y + 1, VIEWPORT));
    assert_eq!(preview.screen(), Screen::Calibrate);
    assert_eq!(preview.overlay(), Overlay::None);
}
