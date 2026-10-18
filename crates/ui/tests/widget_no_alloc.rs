//! Verifies widget layout, paint, and hit passes do not allocate.
//!
//! This lives in its own test binary so the global allocation counter is not
//! disturbed by tests running in parallel.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use common::Region;
use ui::{
    collect_hits, layout, paint_widgets, ButtonProps, HitRegistry, Insets, Kind, LayoutEntry,
    LayoutRect, Length, MonoSurface, Node, Painter, SliderProps, TextCache, Theme, Widget,
    WidgetArena, WidgetKind,
};

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

#[test]
fn the_widget_path_allocates_nothing() {
    let mut widgets = WidgetArena::new();
    widgets
        .push(Widget::new(
            1,
            WidgetKind::Button(ButtonProps {
                label: "Save",
                icon: None,
            }),
        ))
        .unwrap();
    widgets
        .push(Widget::new(
            2,
            WidgetKind::Slider(SliderProps {
                value: 40,
                label: "Level",
            }),
        ))
        .unwrap();

    let mut cache = TextCache::new();
    let nodes = [
        widgets.get(0).unwrap().node(&Theme::WATCH, &mut cache, 200),
        widgets.get(1).unwrap().node(&Theme::WATCH, &mut cache, 200),
    ];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Column {
            spacing: 0,
            cross_align: ui::Align::Start,
        },
        &nodes,
    );
    let mut entries = [LayoutEntry {
        id: 0,
        rect: LayoutRect::new(0, 0, 0, 0),
        clip: Region::new(0, 0, 16, 16),
    }; 4];
    let mut data = vec![0u8; MonoSurface::buffer_len(16, 16)];
    let mut surface = MonoSurface::new(16, 16, &mut data).unwrap();
    let mut registry = HitRegistry::new();
    let viewport = Region::new(0, 0, 16, 16);

    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for _ in 0..4 {
        let count = layout(&root, viewport, Insets::ZERO, &mut cache, &mut entries).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            paint_widgets(&mut painter, &widgets, &entries[..count], &Theme::WATCH);
        }
        collect_hits(&widgets, &entries[..count], &mut registry).unwrap();
        let _ = registry.focus_next(None);
    }
    let after = ALLOCATIONS.load(Ordering::Relaxed);
    assert_eq!(after, before);
}
