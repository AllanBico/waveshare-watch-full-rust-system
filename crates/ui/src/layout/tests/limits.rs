//! Depth and capacity limit tests.

use super::*;

#[test]
fn depth_is_capped_at_thirty_two_levels() {
    let allowed = nest(MAX_DEPTH);
    let mut entries = [EMPTY; 40];
    let mut cache = TextCache::new();
    assert!(layout(
        allowed,
        Region::new(0, 0, 100, 100),
        Insets::ZERO,
        &mut cache,
        &mut entries,
    )
    .is_ok());

    let denied = nest(MAX_DEPTH + 1);
    let mut entries = [EMPTY; 40];
    let mut cache = TextCache::new();
    assert_eq!(
        layout(
            denied,
            Region::new(0, 0, 100, 100),
            Insets::ZERO,
            &mut cache,
            &mut entries,
        ),
        Err(LayoutError::DepthExceeded)
    );
}

#[test]
fn a_small_output_buffer_reports_overflow() {
    let children = [Node::spacer(1, Length::Fill(1), Length::Fill(1))];
    let root = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Stack {
            align: Align::Start,
        },
        &children,
    );
    let mut entries = [EMPTY; 1];
    let mut cache = TextCache::new();
    assert_eq!(
        layout(
            &root,
            Region::new(0, 0, 10, 10),
            Insets::ZERO,
            &mut cache,
            &mut entries,
        ),
        Err(LayoutError::OutputFull)
    );
}

#[test]
fn zero_column_grid_is_rejected() {
    let grid = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Grid {
            columns: 0,
            column_spacing: 0,
            row_spacing: 0,
        },
        &[],
    );
    assert_eq!(layout_error(&grid), LayoutError::InvalidGrid);
}

#[test]
fn multi_child_scroll_is_rejected() {
    let pair = pair();
    let scroll = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Scroll {
            axis: Axis::Vertical,
            offset: 0,
        },
        &pair,
    );
    assert_eq!(layout_error(&scroll), LayoutError::InvalidScroll);
}

#[test]
fn zero_stride_list_is_rejected() {
    let pair = pair();
    let list = Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        Kind::List {
            row_height: 0,
            spacing: 0,
            offset: 0,
        },
        &pair,
    );
    assert_eq!(layout_error(&list), LayoutError::InvalidList);
}

fn pair() -> [Node<'static>; 2] {
    [
        Node::spacer(1, Length::Fill(1), Length::Fill(1)),
        Node::spacer(2, Length::Fill(1), Length::Fill(1)),
    ]
}

fn layout_error(root: &Node<'_>) -> LayoutError {
    let mut entries = [EMPTY; 4];
    let mut cache = TextCache::new();
    layout(
        root,
        Region::new(0, 0, 100, 100),
        Insets::ZERO,
        &mut cache,
        &mut entries,
    )
    .unwrap_err()
}

fn nest(depth: u8) -> &'static Node<'static> {
    if depth == 1 {
        return Box::leak(Box::new(Node::spacer(1, Length::Fill(1), Length::Fill(1))));
    }
    let child = nest(depth - 1);
    let children: &'static [Node<'static>] = Box::leak(Box::new([*child]));
    Box::leak(Box::new(Node::container(
        u16::from(depth),
        Length::Fill(1),
        Length::Fill(1),
        Kind::Column {
            spacing: 0,
            cross_align: Align::Start,
        },
        children,
    )))
}
