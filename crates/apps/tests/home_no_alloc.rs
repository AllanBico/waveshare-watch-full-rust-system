//! Verifies the watch and desk Home render paths do not allocate.
//!
//! This lives in its own test binary so the global allocation counter is not
//! disturbed by tests running in parallel.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use apps::{Home, LogsApp, SettingsApp};
use common::{Cpu1State, DateTime, Mode, Percent, Region, SystemStats, Timestamp};
use settings::{Settings, TimeFormat};
use ui::{GraySurface, MonoSurface, Theme};

struct CountingAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: this is a test-only wrapper around the system allocator. It forwards
// every operation unchanged and only increments a counter.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        // SAFETY: the caller upholds the GlobalAlloc contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the caller upholds the GlobalAlloc contract.
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

const WIDTH: u16 = 410;
const HEIGHT: u16 = 502;

#[test]
fn the_home_render_path_allocates_nothing() {
    let now = DateTime::new(2026, 10, 18, 7, 5, 0).to_timestamp().unwrap();
    let mut watch = Home::new(
        Mode::Watch,
        now,
        Percent::new(80),
        false,
        TimeFormat::Hour24,
    );
    let mut desk = Home::new(Mode::Desk, now, Percent::new(80), false, TimeFormat::Hour24);
    let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    let mut gray_data = vec![0u8; GraySurface::buffer_len(WIDTH, HEIGHT)];
    let mut gray_surface = GraySurface::new(WIDTH, HEIGHT, &mut gray_data).unwrap();
    let viewport = Region::full(WIDTH, HEIGHT);
    let mut next = Timestamp::from_secs(now.as_secs());
    let mut settings_app = SettingsApp::new(Mode::Watch, Settings::new());
    let mut logs_app = LogsApp::new();

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for _ in 0..4 {
        next = Timestamp::from_secs(next.as_secs().saturating_add(60));
        watch.update(next, Percent::new(80), false);
        watch
            .render_frame(&mut surface, &Theme::WATCH, viewport)
            .unwrap();
        desk.update(next, Percent::new(80), false);
        desk.set_stats(SystemStats {
            cpu_percent: Percent::new(7),
            cpu1_percent: Percent::new(11),
            cpu1_state: Cpu1State::Audio,
            cpu_mhz: 240,
            ram_used_kb: 143,
            ram_total_kb: 320,
            psram_used_kb: 1024,
            psram_total_kb: 8192,
        });
        desk.render_frame(&mut gray_surface, &Theme::DESK, viewport)
            .unwrap();
        settings_app
            .render_frame(&mut surface, &Theme::WATCH, viewport)
            .unwrap();
        logs_app
            .render_frame(&mut surface, &Theme::WATCH, viewport)
            .unwrap();
    }
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after, before);
}
