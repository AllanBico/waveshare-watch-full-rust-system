//! Hit testing and focus traversal tests.

use common::Region;

use crate::layout::{layout, Align, Insets, Kind, LayoutEntry, LayoutRect, Length, Node};
use crate::text::TextCache;
use crate::theme::Theme;
use crate::widgets::{
    collect_hits, BadgeProps, ButtonProps, HitRegistry, SliderProps, ToggleProps, Widget,
    WidgetArena, WidgetKind, WidgetState,
};

const SAMPLE: [Widget<'static>; 4] = [
    Widget::new(
        1,
        WidgetKind::Button(ButtonProps {
            label: "One",
            icon: None,
        }),
    ),
    Widget::new(
        2,
        WidgetKind::Toggle(ToggleProps {
            checked: false,
            label: "Off",
        }),
    )
    .with_state(WidgetState::DISABLED),
    Widget::new(
        3,
        WidgetKind::Slider(SliderProps {
            value: 30,
            label: "Level",
        }),
    ),
    Widget::new(4, WidgetKind::Badge(BadgeProps { label: "New" })),
];

fn sample() -> (WidgetArena<'static>, [LayoutEntry; 8], usize) {
    let mut widgets = WidgetArena::new();
    for widget in SAMPLE {
        widgets.push(widget).unwrap();
    }
    let mut cache = TextCache::new();
    let nodes = [
        widgets.get(0).unwrap().node(&Theme::WATCH, &mut cache, 400),
        widgets.get(1).unwrap().node(&Theme::WATCH, &mut cache, 400),
        widgets.get(2).unwrap().node(&Theme::WATCH, &mut cache, 400),
        widgets.get(3).unwrap().node(&Theme::WATCH, &mut cache, 400),
    ];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Column {
            spacing: 0,
            cross_align: Align::Start,
        },
        &nodes,
    );
    let mut entries = [LayoutEntry {
        id: 0,
        rect: LayoutRect::new(0, 0, 0, 0),
        clip: Region::new(0, 0, 0, 0),
    }; 8];
    let count = layout(
        &root,
        Region::full(400, 400),
        Insets::ZERO,
        &mut cache,
        &mut entries,
    )
    .unwrap();
    (widgets, entries, count)
}

#[test]
fn layout_entries_feed_hit_registration() {
    let (widgets, entries, count) = sample();
    let mut registry = HitRegistry::new();
    collect_hits(&widgets, &entries[..count], &mut registry).unwrap();
    assert_eq!(registry.hit(4, 4), Some(1));
    assert_eq!(registry.hit(4, 150), Some(3));
    assert_eq!(registry.hit(4, 300), None);
}

#[test]
fn focus_order_follows_paint_order_and_skips_disabled() {
    let (widgets, entries, count) = sample();
    let mut registry = HitRegistry::new();
    collect_hits(&widgets, &entries[..count], &mut registry).unwrap();

    assert_eq!(registry.focus_next(None), Some(1));
    assert_eq!(registry.focus_next(Some(1)), Some(3));
    assert_eq!(registry.focus_next(Some(3)), Some(1));
    assert_eq!(registry.focus_prev(Some(1)), Some(3));
    assert_eq!(registry.focus_prev(Some(3)), Some(1));
}
