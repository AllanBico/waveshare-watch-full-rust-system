//! Stack, grid, list, scroll, and text layout tests.

use super::*;

#[test]
fn stack_centers_on_both_axes() {
    let children = [Node::spacer(1, Length::Fixed(20), Length::Fixed(10))];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Stack {
            align: Align::Center,
        },
        &children,
    );
    let mut entries = [EMPTY; 4];
    let count = run(
        &root,
        Region::new(0, 0, 100, 50),
        Insets::ZERO,
        &mut entries,
    );
    assert_eq!(find(&entries[..count], 1).rect, rect(40, 20, 20, 10));
}

#[test]
fn safe_area_and_padding_compose() {
    let children = [Node::spacer(1, Length::Fill(1), Length::Fill(1))];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Stack {
            align: Align::Start,
        },
        &children,
    )
    .with_padding(Insets::all(5));
    let mut entries = [EMPTY; 4];
    let count = run(
        &root,
        Region::new(0, 0, 100, 100),
        Insets::all(10),
        &mut entries,
    );
    let entries = &entries[..count];
    assert_eq!(find(entries, 0).rect, rect(10, 10, 80, 80));
    assert_eq!(find(entries, 0).clip, Region::new(10, 10, 80, 80));
    assert_eq!(find(entries, 1).rect, rect(15, 15, 70, 70));
}

#[test]
fn overflowing_rows_are_clipped_to_content() {
    let children = [
        Node::spacer(1, Length::Fixed(60), Length::Fixed(10)),
        Node::spacer(2, Length::Fixed(60), Length::Fixed(10)),
    ];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Row {
            spacing: 0,
            cross_align: Align::Start,
        },
        &children,
    );
    let mut entries = [EMPTY; 4];
    let count = run(&root, Region::new(0, 0, 80, 40), Insets::ZERO, &mut entries);
    let entries = &entries[..count];
    assert_eq!(find(entries, 2).rect, rect(60, 0, 60, 10));
    assert_eq!(find(entries, 2).clip, Region::new(60, 0, 20, 10));
}

#[test]
fn scroll_offsets_content_and_keeps_the_viewport_clip() {
    let inner = [Node::spacer(2, Length::Fill(1), Length::Fixed(200))];
    let content = Node::container(
        1,
        Length::Fill(1),
        Length::Fixed(200),
        Kind::Column {
            spacing: 0,
            cross_align: Align::Start,
        },
        &inner,
    );
    let children = [content];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Scroll {
            axis: Axis::Vertical,
            offset: 30,
        },
        &children,
    );
    let mut entries = [EMPTY; 8];
    let count = run(
        &root,
        Region::new(0, 0, 100, 100),
        Insets::ZERO,
        &mut entries,
    );
    let entries = &entries[..count];
    assert_eq!(find(entries, 1).rect, rect(0, -30, 100, 200));
    assert_eq!(find(entries, 1).clip, Region::new(0, 0, 100, 100));
    assert_eq!(find(entries, 2).rect, rect(0, -30, 100, 200));
}

#[test]
fn list_lays_out_only_visible_rows() {
    let children: Vec<Node> = (0..1_000)
        .map(|id| Node::spacer(id, Length::Fill(1), Length::Fill(1)))
        .collect();
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::List {
            row_height: 20,
            spacing: 0,
            offset: 100,
        },
        &children,
    );
    let mut entries = [EMPTY; 8];
    let count = run(
        &root,
        Region::new(0, 0, 100, 50),
        Insets::ZERO,
        &mut entries,
    );
    assert_eq!(count, 4);
    assert_eq!(find(&entries[..count], 5).rect, rect(0, 0, 100, 20));
    assert_eq!(find(&entries[..count], 6).rect, rect(0, 20, 100, 20));
    assert_eq!(find(&entries[..count], 7).rect, rect(0, 40, 100, 20));
}

#[test]
fn grid_uses_uniform_tracks_and_row_heights() {
    let children = [
        Node::spacer(1, Length::Fill(1), Length::Fixed(10)),
        Node::spacer(2, Length::Fill(1), Length::Fixed(10)),
        Node::spacer(3, Length::Fill(1), Length::Fixed(10)),
    ];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Grid {
            columns: 2,
            column_spacing: 10,
            row_spacing: 5,
        },
        &children,
    );
    let mut entries = [EMPTY; 8];
    let count = run(
        &root,
        Region::new(0, 0, 100, 100),
        Insets::ZERO,
        &mut entries,
    );
    let entries = &entries[..count];
    assert_eq!(find(entries, 1).rect, rect(0, 0, 45, 10));
    assert_eq!(find(entries, 2).rect, rect(55, 0, 45, 10));
    assert_eq!(find(entries, 3).rect, rect(0, 15, 45, 10));
}

#[test]
fn text_uses_measurement_and_the_cache() {
    let style = Theme::WATCH.type_scale.caption;
    let text = "hello world again";
    let expected = measure_uncached(text, &style, 100);
    let children = [Node::text(1, Length::Fit, Length::Fit, text, style)];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Column {
            spacing: 0,
            cross_align: Align::Start,
        },
        &children,
    );
    let mut entries = [EMPTY; 4];
    let mut cache = TextCache::new();
    let count = layout(
        &root,
        Region::new(0, 0, 100, 100),
        Insets::ZERO,
        &mut cache,
        &mut entries,
    )
    .unwrap();
    let text_rect = find(&entries[..count], 1).rect;
    assert_eq!(text_rect, rect(0, 0, expected.width, expected.height));
    assert!(!cache.is_empty());
}
