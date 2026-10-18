//! State rendering and purity tests.

use crate::assets::IconId;
use crate::text::TextCache;
use crate::theme::Theme;
use crate::widgets::{ButtonProps, Widget, WidgetKind, WidgetState};

use super::{render_desk, render_watch, SAMPLE_KINDS};

const WIDTH: u16 = 280;

fn button() -> WidgetKind<'static> {
    WidgetKind::Button(ButtonProps {
        label: "Continue",
        icon: Some(IconId::Check),
    })
}

#[test]
fn each_state_changes_the_rendered_button() {
    let normal = render_watch(button(), WidgetState::NORMAL, WIDTH, 88).0;
    for state in [
        WidgetState::FOCUSED,
        WidgetState::PRESSED,
        WidgetState::DISABLED,
        WidgetState::SELECTED,
        WidgetState::ERROR,
    ] {
        let rendered = render_watch(button(), state, WIDTH, 88).0;
        assert_ne!(rendered, normal, "state {state:?} did not change the frame");
    }
}

#[test]
fn combinations_render_independently() {
    let focused = WidgetState::NORMAL.with(WidgetState::FOCUSED);
    let focused_selected = focused.with(WidgetState::SELECTED);
    assert_ne!(
        render_watch(button(), focused, WIDTH, 88).0,
        render_watch(button(), focused_selected, WIDTH, 88).0
    );
}

#[test]
fn painting_the_same_props_is_idempotent() {
    let first = render_watch(button(), WidgetState::NORMAL, WIDTH, 88);
    let second = render_watch(button(), WidgetState::NORMAL, WIDTH, 88);
    assert_eq!(first, second);
    let first = render_desk(button(), WidgetState::NORMAL, WIDTH, 88);
    let second = render_desk(button(), WidgetState::NORMAL, WIDTH, 88);
    assert_eq!(first, second);
}

#[test]
fn props_and_state_drive_the_fingerprint() {
    let first = Widget::new(1, button());
    let mut other = Widget::new(
        1,
        WidgetKind::Button(ButtonProps {
            label: "Cancel",
            icon: None,
        }),
    );
    assert_ne!(first.fingerprint(), other.fingerprint());
    other.kind = button();
    assert_eq!(first.fingerprint(), other.fingerprint());
    other.state = WidgetState::PRESSED;
    assert_ne!(first.fingerprint(), other.fingerprint());
}

#[test]
fn measure_does_not_mutate_the_widget() {
    let widget = Widget::new(1, button());
    let mut cache = TextCache::new();
    let _ = widget.node(&Theme::WATCH, &mut cache, WIDTH);
    assert_eq!(widget, Widget::new(1, button()));
}

#[test]
fn every_kind_renders_in_both_modes() {
    for (index, kind) in SAMPLE_KINDS.into_iter().enumerate() {
        let (hash, lit) = render_watch(kind, WidgetState::NORMAL, 280, 160);
        assert!(lit > 0, "watch kind {index} rendered no pixels");
        assert_ne!(hash, 0);
        assert_ne!(render_desk(kind, WidgetState::NORMAL, 280, 160), 0);
    }
}

#[test]
fn widget_kinds_are_plain_data() {
    fn plain<T: Copy + Eq + core::hash::Hash>() {}
    plain::<WidgetKind<'static>>();
    plain::<WidgetState>();
}
