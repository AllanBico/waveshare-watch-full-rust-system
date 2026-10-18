//! Verifies the scene, damage, paint, and flush passes do not allocate.
//!
//! The counting allocator is test-only instrumentation; the `ui` crate itself
//! forbids unsafe code.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use common::Region;
use hal::Display;
use ui::{flush, paint_damage, Luma, MonoSurface, Scene, SceneArena, SceneNode, WidgetId};

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

struct NullDisplay;

impl Display for NullDisplay {
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

    fn write_region(&mut self, _region: Region, _pixels: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn arena(id: WidgetId, x: i32) -> SceneArena {
    let mut arena = SceneArena::new();
    arena
        .push(SceneNode::new(id, ui::LayoutRect::new(x, 4, 8, 8), 1))
        .unwrap();
    arena
}

#[test]
fn the_render_path_allocates_nothing() {
    let mut data = vec![0u8; MonoSurface::buffer_len(16, 16)];
    let mut surface = MonoSurface::new(16, 16, &mut data).unwrap();
    let mut scene = Scene::new();
    let first = arena(1, 4);
    let second = arena(1, 6);
    let mut staging = [0u8; 64];
    let mut display = NullDisplay;
    let viewport = Region::new(0, 0, 16, 16);

    scene.diff(&first, viewport);
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for _ in 0..4 {
        let damage = scene.diff(&second, viewport);
        paint_damage(&mut surface, &damage, Luma::BLACK, |_, _| {});
        flush(&mut display, &surface, &damage, &mut staging).unwrap();
    }
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after, before);
}
