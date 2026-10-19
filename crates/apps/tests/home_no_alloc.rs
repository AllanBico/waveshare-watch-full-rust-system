//! Verifies the watch Home render path does not allocate.
//!
//! This lives in its own test binary so the global allocation counter is not
//! disturbed by tests running in parallel.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use apps::Home;
use common::{DateTime, Mode, Percent, Region, Timestamp};
use settings::TimeFormat;
use ui::{MonoSurface, Theme};

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
fn the_watch_home_render_path_allocates_nothing() {
    let now = DateTime::new(2026, 10, 18, 7, 5, 0).to_timestamp().unwrap();
    let mut home = Home::new(
        Mode::Watch,
        now,
        Percent::new(80),
        false,
        TimeFormat::Hour24,
    );
    let mut data = vec![0u8; MonoSurface::buffer_len(WIDTH, HEIGHT)];
    let mut surface = MonoSurface::new(WIDTH, HEIGHT, &mut data).unwrap();
    let viewport = Region::full(WIDTH, HEIGHT);
    let mut next = Timestamp::from_secs(now.as_secs());

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for _ in 0..4 {
        next = Timestamp::from_secs(next.as_secs().saturating_add(60));
        home.update(next, Percent::new(80), false);
        home.render_frame(&mut surface, &Theme::WATCH, viewport)
            .unwrap();
    }
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after, before);
}
