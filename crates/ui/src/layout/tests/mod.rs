//! Host tests for integer layout.

mod containers;
mod golden;
mod limits;
mod linear;

use common::Region;

use super::*;
use crate::text::{measure_uncached, TextCache};
use crate::theme::Theme;

const EMPTY: LayoutEntry = LayoutEntry {
    id: 0,
    rect: LayoutRect::new(0, 0, 0, 0),
    clip: Region::new(0, 0, 0, 0),
};

fn run(root: &Node<'_>, viewport: Region, safe: Insets, entries: &mut [LayoutEntry]) -> usize {
    let mut cache = TextCache::new();
    layout(root, viewport, safe, &mut cache, entries).unwrap()
}

fn find(entries: &[LayoutEntry], id: NodeId) -> &LayoutEntry {
    entries.iter().find(|entry| entry.id == id).unwrap()
}

fn rect(x: i32, y: i32, width: u16, height: u16) -> LayoutRect {
    LayoutRect::new(x, y, width, height)
}
