//! Scene, damage, and pacing tests.

use common::Region;

use super::*;
use crate::layout::LayoutRect;

const VIEWPORT: Region = Region::new(0, 0, 100, 100);

fn node(id: WidgetId, x: i32, y: i32, width: u16, height: u16, fp: u64) -> SceneNode {
    SceneNode::new(id, LayoutRect::new(x, y, width, height), fp)
}

fn arena(nodes: &[SceneNode]) -> SceneArena {
    let mut arena = SceneArena::new();
    for &node in nodes {
        arena.push(node).unwrap();
    }
    arena
}

#[test]
fn first_diff_is_a_full_redraw() {
    let mut scene = Scene::new();
    let damage = scene.diff(&arena(&[node(1, 10, 10, 20, 20, 1)]), VIEWPORT);
    assert!(damage.is_full());
    assert_eq!(damage.regions(), &[VIEWPORT]);
    assert!(scene.has_frame());
}

#[test]
fn a_pending_full_redraw_is_consumed_once() {
    let mut scene = Scene::new();
    scene.request_full(RedrawReason::Boot);
    let frame = arena(&[node(1, 10, 10, 20, 20, 1)]);
    assert!(scene.diff(&frame, VIEWPORT).is_full());
    assert!(!scene.diff(&frame, VIEWPORT).is_full());
}

#[test]
fn move_dirties_the_old_and_new_rects() {
    let mut scene = Scene::new();
    let old = [node(1, 10, 10, 20, 20, 1)];
    scene.diff(&arena(&old), VIEWPORT);
    let new = [node(1, 12, 10, 20, 20, 1)];
    let damage = scene.diff(&arena(&new), VIEWPORT);
    assert!(!damage.is_full());
    assert_eq!(damage.regions(), &[Region::new(10, 10, 22, 20)]);
    assert!(damage.total_pixels() < VIEWPORT.pixel_count());
}

#[test]
fn insert_and_remove_dirty_only_the_changed_node() {
    let mut scene = Scene::new();
    let first = [node(1, 10, 10, 20, 20, 1)];
    scene.diff(&arena(&first), VIEWPORT);

    let inserted = [node(1, 10, 10, 20, 20, 1), node(2, 40, 40, 10, 10, 1)];
    let damage = scene.diff(&arena(&inserted), VIEWPORT);
    assert_eq!(damage.regions(), &[Region::new(40, 40, 10, 10)]);

    let damage = scene.diff(&arena(&first), VIEWPORT);
    assert_eq!(damage.regions(), &[Region::new(40, 40, 10, 10)]);
}

#[test]
fn a_moving_container_collapses_to_one_rect() {
    let mut scene = Scene::new();
    let old = [
        node(1, 10, 10, 40, 40, 1),
        node(2, 12, 12, 10, 10, 1),
        node(3, 30, 12, 10, 10, 1),
    ];
    scene.diff(&arena(&old), VIEWPORT);
    let new = [
        node(1, 14, 10, 40, 40, 1),
        node(2, 16, 12, 10, 10, 1),
        node(3, 34, 12, 10, 10, 1),
    ];
    let damage = scene.diff(&arena(&new), VIEWPORT);
    assert_eq!(damage.regions(), &[Region::new(10, 10, 44, 40)]);
}

#[test]
fn theme_switch_forces_everything_dirty() {
    let mut scene = Scene::new();
    let frame = [node(1, 10, 10, 20, 20, 1), node(2, 40, 40, 10, 10, 1)];
    scene.diff(&arena(&frame), VIEWPORT);
    scene.request_full(RedrawReason::ThemeSwitch);
    let damage = scene.diff(&arena(&frame), VIEWPORT);
    assert!(damage.is_full());
    assert_eq!(damage.regions(), &[VIEWPORT]);
}

#[test]
fn dirty_rect_cap_holds_for_many_scattered_changes() {
    let mut scene = Scene::new();
    let mut old = [SceneNode::new(0, LayoutRect::new(0, 0, 0, 0), 0); 16];
    let mut new = old;
    for (index, node) in old.iter_mut().enumerate() {
        let x = (index % 4) as i32 * 20;
        let y = (index / 4) as i32 * 20;
        *node = SceneNode::new(index as WidgetId, LayoutRect::new(x, y, 2, 2), 1);
        new[index] = SceneNode::new(index as WidgetId, LayoutRect::new(x + 4, y, 2, 2), 1);
    }
    scene.diff(&arena(&old), VIEWPORT);
    let damage = scene.diff(&arena(&new), VIEWPORT);
    assert!(!damage.is_full());
    assert!(damage.regions().len() <= MAX_DAMAGE_RECTS);
}

#[test]
fn damage_merges_touching_regions_but_keeps_disjoint_ones() {
    let mut damage = Damage::new();
    damage.add(Region::new(0, 0, 10, 10));
    damage.add(Region::new(10, 0, 10, 10));
    assert_eq!(damage.regions(), &[Region::new(0, 0, 20, 10)]);
    damage.add(Region::new(80, 80, 5, 5));
    assert_eq!(damage.regions().len(), 2);
}

#[test]
fn frame_pacer_caps_animated_frames() {
    let mut pacer = FramePacer::new();
    assert!(pacer.allow(0, true));
    assert!(!pacer.allow(8, true));
    assert!(pacer.allow(16, true));
    assert!(pacer.allow(17, false));
    assert!(pacer.allow(18, false));
}
