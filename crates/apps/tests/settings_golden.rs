//! Settings layout golden frames for watch and desk.

use apps::SettingsApp;
use common::{Mode, Region};
use settings::Settings;
use ui::{Damage, GraySurface, MonoSurface, Surface, Theme};

const WIDTH: u16 = 410;
const HEIGHT: u16 = 502;

fn render_watch(app: &mut SettingsApp) -> (Damage, u32, u64) {
    let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    let damage = app
        .render_frame(&mut surface, &Theme::WATCH, Region::full(WIDTH, HEIGHT))
        .unwrap();
    (damage, surface.lit_pixels(), hash(&surface))
}

fn render_desk(app: &mut SettingsApp) -> (Damage, u32, u64) {
    let mut data = vec![0u8; GraySurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = GraySurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    let damage = app
        .render_frame(&mut surface, &Theme::DESK, Region::full(WIDTH, HEIGHT))
        .unwrap();
    let mut lit = 0u32;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if surface.pixel(x, y).raw() != 0 {
                lit += 1;
            }
        }
    }
    (damage, lit, hash(&surface))
}

fn hash(surface: &impl Surface) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            hash ^= u64::from(surface.pixel(x, y).raw());
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    hash
}

#[test]
fn watch_settings_golden_frame_is_stable() {
    let mut app = SettingsApp::new(Mode::Watch, Settings::new());
    let (damage, lit, hash) = render_watch(&mut app);
    assert!(damage.is_full());
    assert_eq!((hash, lit), (0xf4ab_3cc3_9c14_d463, 11992));
}

#[test]
fn desk_settings_golden_frame_is_stable() {
    let mut app = SettingsApp::new(Mode::Desk, Settings::new());
    let (damage, lit, hash) = render_desk(&mut app);
    assert!(damage.is_full());
    assert_eq!((hash, lit), (0xcbdd_732a_5f1f_fa87, 140231));
}
