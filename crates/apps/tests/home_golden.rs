//! Watch and desk Home golden frame, idle damage, and pixel-budget tests.

use apps::Home;
use common::{DateTime, Mode, Percent, Region, Timestamp};
use settings::TimeFormat;
use ui::{Damage, GraySurface, MonoSurface, Surface, Theme};

const WIDTH: u16 = 410;
const HEIGHT: u16 = 502;

fn at(hour: u8, minute: u8) -> Timestamp {
    DateTime::new(2026, 10, 18, hour, minute, 0)
        .to_timestamp()
        .unwrap()
}

fn home() -> Home {
    Home::new(
        Mode::Watch,
        at(7, 5),
        Percent::new(80),
        false,
        TimeFormat::Hour24,
    )
}

fn desk_home() -> Home {
    Home::new(
        Mode::Desk,
        at(7, 5),
        Percent::new(80),
        false,
        TimeFormat::Hour24,
    )
}

fn render(home: &mut Home) -> (Damage, u32, u64) {
    let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    let damage = home
        .render_frame(&mut surface, &Theme::WATCH, Region::full(WIDTH, HEIGHT))
        .unwrap();
    (damage, surface.lit_pixels(), hash(&surface))
}

fn render_desk(home: &mut Home) -> (Damage, u32, u64) {
    let mut data = vec![0u8; GraySurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = GraySurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    let damage = home
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
fn watch_home_golden_frame_is_stable() {
    let (damage, lit, hash) = render(&mut home());
    assert!(damage.is_full());
    assert_eq!((hash, lit), (0xc0b4_71cf_80af_cc62, 4255));
}

#[test]
fn desk_home_golden_frame_is_stable() {
    let (damage, lit, hash) = render_desk(&mut desk_home());
    assert!(damage.is_full());
    assert_eq!((hash, lit), (0xdc06_b6ed_1fca_8ac5, 6590));
}

#[test]
fn desk_idle_home_reports_no_damage() {
    let mut home = desk_home();
    render_desk(&mut home);
    let (damage, lit, _) = render_desk(&mut home);
    assert!(damage.is_empty());
    assert_eq!(lit, 0);
}

#[test]
fn desk_charging_repaints_only_the_changed_frame() {
    let mut home = desk_home();
    render_desk(&mut home);
    assert!(home.update(at(7, 5), Percent::new(80), true));
    let (damage, lit, _) = render_desk(&mut home);
    assert!(!damage.is_empty());
    assert!(!damage.is_full());
    assert!(lit > 0);
}

#[test]
fn watch_alarm_frame_is_stable() {
    let mut home = home();
    home.set_alarm(Some(at(7, 5)));
    let (_, _, hash) = render(&mut home);
    assert_eq!(hash, 0xc2e9_8240_92b7_10eb);
}

#[test]
fn watch_home_stays_inside_the_lit_pixel_budget() {
    let (_, lit, _) = render(&mut home());
    let area = u32::from(WIDTH) * u32::from(HEIGHT);
    assert!(lit * 100 <= area * 5, "lit {lit} of {area} pixels");
}

#[test]
fn idle_home_reports_no_damage() {
    let mut home = home();
    render(&mut home);
    let (damage, lit, _) = render(&mut home);
    assert!(damage.is_empty());
    assert_eq!(lit, 0);
}

#[test]
fn a_new_minute_repaints_only_the_changed_frame() {
    let mut home = home();
    render(&mut home);
    assert!(home.update(at(7, 6), Percent::new(80), false));
    let (damage, lit, _) = render(&mut home);
    assert!(!damage.is_empty());
    assert!(!damage.is_full());
    assert!(lit > 0);
}
