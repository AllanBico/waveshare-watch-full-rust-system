//! Host tests for the hardware preview.

use common::{Mode, Percent, Region, Timestamp};
use ui::{GraySurface, MonoSurface, Theme};

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
