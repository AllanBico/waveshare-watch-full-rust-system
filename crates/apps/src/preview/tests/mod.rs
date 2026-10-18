//! Host tests for the hardware preview.

use common::{Mode, Percent, Region, Timestamp};
use ui::{GraySurface, MonoSurface, Surface as _, Theme};

use super::*;

mod calibration;

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
    assert!(preview.back());
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
    assert!(preview.back());
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

#[test]
fn preview_paints_only_inside_the_panel_shape() {
    let mut mono = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut watch = watch_surface(&mut mono);
    let mut watch_preview = preview(Mode::Watch);
    watch_preview
        .render(&mut watch, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert_outside_shape(&watch, Theme::WATCH.shape);
    watch_preview.open_drawer();
    watch_preview
        .render(&mut watch, &Theme::WATCH, VIEWPORT)
        .unwrap();
    assert_outside_shape(&watch, Theme::WATCH.shape);

    let mut gray = vec![0u8; GraySurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut desk = GraySurface::new(VIEWPORT.width, VIEWPORT.height, &mut gray).unwrap();
    let mut desk_preview = preview(Mode::Desk);
    desk_preview
        .render(&mut desk, &Theme::DESK, VIEWPORT)
        .unwrap();
    assert_outside_shape(&desk, Theme::DESK.shape);
}

fn assert_outside_shape(surface: &impl ui::Surface, shape: ui::ScreenShape) {
    for y in 0..surface.height() {
        for x in 0..surface.width() {
            if !shape.contains(x, y, surface.width(), surface.height()) {
                assert_eq!(
                    surface.pixel(x, y),
                    ui::Luma::BLACK,
                    "lit pixel outside the shape at {x},{y}"
                );
            }
        }
    }
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
fn the_drawer_closes_with_back() {
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
    assert!(preview.back());
    assert_eq!(preview.overlay(), Overlay::None);
}

#[test]
fn every_gallery_page_fits_above_the_next_control() {
    const EXPECTED: [usize; PAGES as usize] = [4, 4, 4, 4, 4, 5, 4, 3, 2, 1, 1];
    let mut data = vec![0u8; MonoSurface::buffer_len(VIEWPORT.width, VIEWPORT.height)];
    let mut surface = watch_surface(&mut data);
    let mut preview = preview(Mode::Watch);
    preview.screen = Screen::Gallery;
    for page in 0..PAGES {
        preview.gallery_page = page;
        preview.invalidate();
        preview
            .render(&mut surface, &Theme::WATCH, VIEWPORT)
            .unwrap();
        let next = hit(&preview, 303);
        let mut found = 0;
        for id in 310..=314 {
            let Some(target) = preview.hits.targets().iter().find(|target| target.id == id) else {
                continue;
            };
            found += 1;
            assert!(
                target.rect.bottom() <= next.rect.y,
                "page {page} widget {id} overlaps the footer"
            );
        }
        assert_eq!(found, EXPECTED[usize::from(page)], "page {page} widgets");
    }
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
