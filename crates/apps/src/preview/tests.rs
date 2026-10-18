//! Host tests for the hardware preview.

use common::{Mode, Percent, Region, Timestamp};
use ui::{GraySurface, MonoSurface, Surface as _, Theme};

use super::*;

const VIEWPORT: Region = Region::new(0, 0, 410, 502);
const NOW: Timestamp = Timestamp::from_secs(1_760_000_000);

fn preview(mode: Mode) -> Preview {
    Preview::new(mode, NOW, Percent::new(80), true)
}

fn watch_surface<'a>(data: &'a mut [u8]) -> MonoSurface<'a> {
    MonoSurface::new(VIEWPORT.width, VIEWPORT.height, data).unwrap()
}

#[test]
fn watch_home_boots_full_and_stays_under_the_pixel_budget() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    let damage = preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert!(damage.is_full());
    let lit = surface.lit_pixels();
    let budget = VIEWPORT.pixel_count() * 5 / 100;
    assert!(lit < budget, "watch home lit {lit} pixels, budget {budget}");
    assert!(!preview.needs_render());
}

#[test]
fn an_unchanged_frame_produces_no_damage() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let damage = preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert!(damage.is_empty());
}

#[test]
fn a_new_minute_damages_only_the_clock() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert!(preview.update(
        Timestamp::from_secs(NOW.as_secs() + 60),
        Percent::new(80),
        true
    ));
    let damage = preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert!(!damage.is_full());
    assert!(damage.total_pixels() < VIEWPORT.pixel_count() / 2);
}

#[test]
fn the_drawer_opens_without_a_full_redraw_and_launches_placeholders() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    preview.open_drawer();
    let damage = preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert!(!damage.is_full());
    let target = hit(&preview, drawer::SETTINGS_ID);
    assert!(preview.tap(target.rect.x + 1, target.rect.y + 1, VIEWPORT));
    assert_eq!(preview.screen(), Screen::Settings);
    assert_eq!(preview.overlay(), Overlay::None);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let back = hit(&preview, 401);
    assert!(preview.tap(back.rect.x + 1, back.rect.y + 1, VIEWPORT));
    assert_eq!(preview.screen(), Screen::Home);
}

#[test]
fn quick_settings_toggle_and_slider_update_locally() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    preview.open_quick_settings();
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let wifi = hit(&preview, quick::WIFI_ID);
    let was_on = preview.quick().wifi;
    assert!(preview.tap(wifi.rect.x + 1, wifi.rect.y + 1, VIEWPORT));
    assert_eq!(preview.quick().wifi, !was_on);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let brightness = hit(&preview, quick::BRIGHTNESS_ID);
    let x = brightness.rect.x + brightness.rect.width / 4;
    assert!(preview.tap(x, brightness.rect.y + 1, VIEWPORT));
    assert!(preview.quick().brightness < 40);
}

#[test]
fn the_gallery_cycles_pages_and_returns_home() {
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
    let gallery = hit(&preview, drawer::GALLERY_ID);
    assert!(preview.tap(gallery.rect.x + 1, gallery.rect.y + 1, VIEWPORT));
    assert_eq!(preview.screen(), Screen::Gallery);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let next = hit(&preview, 303);
    assert!(preview.tap(next.rect.x + 1, next.rect.y + 1, VIEWPORT));
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let back = hit(&preview, 301);
    assert!(preview.tap(back.rect.x + 1, back.rect.y + 1, VIEWPORT));
    assert_eq!(preview.screen(), Screen::Home);
}

#[test]
fn desk_home_renders_in_grayscale() {
    let mut data = vec![0u8; GraySurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = GraySurface::new(VIEWPORT.width, VIEWPORT.height, &mut data).unwrap();
    let mut preview = preview(Mode::Desk);
    let damage = preview
        .render(&mut surface, &Theme::DESK, VIEWPORT)
        .unwrap();
    assert!(damage.is_full());
    assert!(data.iter().any(|&pixel| pixel != 0));
}

fn hit(preview: &Preview, id: u16) -> ui::HitTarget {
    *preview
        .hits
        .targets()
        .iter()
        .find(|target| target.id == id)
        .unwrap()
}

#[test]
fn home_complications_fit_inside_the_panel() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview
        .render(&mut surface, &Theme::WATCH, VIEWPORT)
        .unwrap();
    let clock = hit(&preview, crate::home::CLOCK_ID);
    let battery = hit(&preview, crate::home::BATTERY_ID);
    assert!(clock.rect.width > 120);
    assert!(clock.rect.right() <= VIEWPORT.width);
    assert!(battery.rect.right() <= VIEWPORT.width);
    assert!(battery.rect.x > clock.rect.x);
}

#[test]
fn the_drawer_closes_from_its_close_button() {
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
    let close = hit(&preview, 102);
    assert!(preview.tap(close.rect.x + 1, close.rect.y + 1, VIEWPORT));
    assert_eq!(preview.overlay(), Overlay::None);
}

#[test]
fn every_gallery_page_renders_and_keeps_the_next_control() {
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview.screen = Screen::Gallery;
    for _ in 0..PAGES {
        preview.invalidate();
        preview
            .render(&mut surface, &Theme::WATCH, VIEWPORT)
            .unwrap();
        let next = hit(&preview, 303);
        assert!(next.rect.bottom() <= VIEWPORT.height);
        assert!(preview.tap(next.rect.x + 1, next.rect.y + 1, VIEWPORT));
    }
    assert_eq!(preview.screen(), Screen::Gallery);
}

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

    let back = hit(&preview, calibrate::BACK_ID);
    assert!(preview.tap(back.rect.x + 1, back.rect.y + 1, VIEWPORT));
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
