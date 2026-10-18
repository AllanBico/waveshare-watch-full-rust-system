//! Row and column layout tests.

use super::*;

#[test]
fn row_fill_splits_remainder_deterministically() {
    let children = [
        Node::spacer(1, Length::Fill(1), Length::Fill(1)),
        Node::spacer(2, Length::Fill(2), Length::Fill(1)),
        Node::spacer(3, Length::Fill(3), Length::Fill(1)),
    ];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Row {
            spacing: 4,
            cross_align: Align::Start,
        },
        &children,
    );
    let mut entries = [EMPTY; 8];
    let count = run(
        &root,
        Region::new(0, 0, 101, 20),
        Insets::ZERO,
        &mut entries,
    );
    assert_eq!(count, 4);
    let entries = &entries[..count];
    assert_eq!(find(entries, 1).rect, rect(0, 0, 15, 20));
    assert_eq!(find(entries, 2).rect, rect(19, 0, 31, 20));
    assert_eq!(find(entries, 3).rect, rect(54, 0, 47, 20));
}

#[test]
fn fixed_percent_and_fit_resolve_in_a_row() {
    let style = Theme::WATCH.type_scale.caption;
    let fit = measure_uncached("Hi", &style, 0).width;
    let children = [
        Node::spacer(1, Length::Fixed(20), Length::Fixed(10)),
        Node::spacer(2, Length::Percent(50), Length::Fixed(10)),
        Node::text(3, Length::Fit, Length::Fit, "Hi", style),
    ];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fixed(30),
        Kind::Row {
            spacing: 0,
            cross_align: Align::Start,
        },
        &children,
    );
    let mut entries = [EMPTY; 8];
    let count = run(
        &root,
        Region::new(0, 0, 200, 30),
        Insets::ZERO,
        &mut entries,
    );
    let entries = &entries[..count];
    assert_eq!(find(entries, 1).rect, rect(0, 0, 20, 10));
    assert_eq!(find(entries, 2).rect, rect(20, 0, 100, 10));
    assert_eq!(find(entries, 3).rect, rect(120, 0, fit, 28));
}

#[test]
fn fills_grow_and_shrink_with_the_viewport() {
    let children = [
        Node::spacer(1, Length::Fill(1), Length::Fill(1)),
        Node::spacer(2, Length::Fill(2), Length::Fill(1)),
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
    let count = run(
        &root,
        Region::new(0, 0, 100, 10),
        Insets::ZERO,
        &mut entries,
    );
    assert_eq!(find(&entries[..count], 1).rect.width, 33);
    assert_eq!(find(&entries[..count], 2).rect.width, 67);
    let count = run(
        &root,
        Region::new(0, 0, 200, 10),
        Insets::ZERO,
        &mut entries,
    );
    assert_eq!(find(&entries[..count], 1).rect.width, 66);
    assert_eq!(find(&entries[..count], 2).rect.width, 134);
}

#[test]
fn column_fill_consumes_the_height_remainder() {
    let children = [
        Node::spacer(1, Length::Fill(1), Length::Fill(1)),
        Node::spacer(2, Length::Fill(1), Length::Fixed(20)),
        Node::spacer(3, Length::Fill(1), Length::Fill(2)),
    ];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Column {
            spacing: 10,
            cross_align: Align::Start,
        },
        &children,
    );
    let mut entries = [EMPTY; 8];
    let count = run(
        &root,
        Region::new(0, 0, 100, 120),
        Insets::ZERO,
        &mut entries,
    );
    let entries = &entries[..count];
    assert_eq!(find(entries, 1).rect, rect(0, 0, 100, 26));
    assert_eq!(find(entries, 2).rect, rect(0, 36, 100, 20));
    assert_eq!(find(entries, 3).rect, rect(0, 66, 100, 54));
}
