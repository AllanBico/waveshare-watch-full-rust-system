use common::{TouchPoint, TouchReport};
use shell::{
    EdgeGesture, GestureConfig, GestureOutcome, GesturePhase, InputRouter, Swipe, SwipeDirection,
    SwipeOrigin,
};

const WIDTH: u16 = 410;
const HEIGHT: u16 = 502;

fn point(x: u16, y: u16) -> TouchPoint {
    TouchPoint::new(1, x, y)
}

fn gesture() -> EdgeGesture {
    EdgeGesture::new(GestureConfig::default())
}

fn swipe(
    origin: SwipeOrigin,
    direction: SwipeDirection,
    distance_px: u16,
    velocity_px_per_s: i32,
) -> GestureOutcome {
    GestureOutcome::Swipe(Swipe {
        origin,
        direction,
        distance_px,
        velocity_px_per_s,
    })
}

#[test]
fn short_press_is_a_tap() {
    let mut gesture = gesture();
    gesture.begin(point(200, 250), 0, WIDTH, HEIGHT);
    assert_eq!(gesture.phase(), GesturePhase::Pressed);
    assert_eq!(gesture.end(150), GestureOutcome::Tap { x: 200, y: 250 });
    assert_eq!(gesture.phase(), GesturePhase::Idle);
}

#[test]
fn movement_inside_slop_is_still_a_tap() {
    let mut gesture = gesture();
    gesture.begin(point(200, 250), 0, WIDTH, HEIGHT);
    gesture.update(point(205, 260), 100);
    assert_eq!(gesture.phase(), GesturePhase::Pressed);
    assert_eq!(gesture.end(150), GestureOutcome::Tap { x: 205, y: 260 });
}

#[test]
fn slow_tap_outside_the_tap_window_is_cancelled() {
    let mut gesture = gesture();
    gesture.begin(point(200, 250), 0, WIDTH, HEIGHT);
    assert_eq!(gesture.end(400), GestureOutcome::Cancelled);
}

#[test]
fn bottom_edge_swipe_up_opens_the_drawer() {
    let mut gesture = gesture();
    gesture.begin(point(200, 500), 0, WIDTH, HEIGHT);
    gesture.update(point(200, 380), 50);
    assert_eq!(
        gesture.end(100),
        swipe(SwipeOrigin::BottomEdge, SwipeDirection::Up, 120, -1_200)
    );
}

#[test]
fn top_edge_swipe_down_opens_quick_settings() {
    let mut gesture = gesture();
    gesture.begin(point(200, 5), 0, WIDTH, HEIGHT);
    gesture.update(point(200, 140), 50);
    assert_eq!(
        gesture.end(100),
        swipe(SwipeOrigin::TopEdge, SwipeDirection::Down, 135, 1_350)
    );
}

#[test]
fn content_swipe_is_tagged_as_content() {
    let mut gesture = gesture();
    gesture.begin(point(200, 250), 0, WIDTH, HEIGHT);
    gesture.update(point(200, 100), 50);
    assert_eq!(
        gesture.end(100),
        swipe(SwipeOrigin::Content, SwipeDirection::Up, 150, -1_500)
    );
}

#[test]
fn horizontal_drag_is_cancelled() {
    let mut gesture = gesture();
    gesture.begin(point(100, 250), 0, WIDTH, HEIGHT);
    gesture.update(point(160, 255), 50);
    assert_eq!(gesture.phase(), GesturePhase::Cancelled);
    assert_eq!(gesture.end(100), GestureOutcome::Cancelled);
}

#[test]
fn short_slow_drag_is_cancelled() {
    let mut gesture = gesture();
    gesture.begin(point(200, 250), 0, WIDTH, HEIGHT);
    gesture.update(point(200, 280), 100);
    assert_eq!(gesture.end(1_000), GestureOutcome::Cancelled);
}

#[test]
fn short_fast_drag_qualifies_by_velocity() {
    let mut gesture = gesture();
    gesture.begin(point(200, 250), 0, WIDTH, HEIGHT);
    gesture.update(point(200, 280), 10);
    assert_eq!(
        gesture.end(20),
        swipe(SwipeOrigin::Content, SwipeDirection::Down, 30, 1_500)
    );
}

#[test]
fn exact_threshold_qualifies_without_velocity() {
    let mut gesture = gesture();
    gesture.begin(point(200, 500), 0, WIDTH, HEIGHT);
    gesture.update(point(200, 404), 500);
    assert_eq!(
        gesture.end(1_000),
        swipe(SwipeOrigin::BottomEdge, SwipeDirection::Up, 96, -96)
    );
}

#[test]
fn cancel_drops_the_touch_without_an_outcome() {
    let mut gesture = gesture();
    gesture.begin(point(200, 500), 0, WIDTH, HEIGHT);
    gesture.cancel();
    assert_eq!(gesture.end(100), GestureOutcome::None);
}

#[test]
fn input_router_tracks_the_first_touch_id() {
    let mut router = InputRouter::new(GestureConfig::default());
    let start = TouchReport::new(&[TouchPoint::new(9, 200, 500)]);
    let moved = TouchReport::new(&[TouchPoint::new(9, 200, 350), TouchPoint::new(10, 300, 300)]);

    assert_eq!(
        router.handle(&start, 0, WIDTH, HEIGHT),
        GestureOutcome::None
    );
    assert_eq!(router.gesture().phase(), GesturePhase::Pressed);
    assert_eq!(
        router.handle(&moved, 50, WIDTH, HEIGHT),
        GestureOutcome::None
    );
    assert_eq!(router.gesture().phase(), GesturePhase::Tracking);
    assert_eq!(
        router.handle(&TouchReport::empty(), 100, WIDTH, HEIGHT),
        swipe(SwipeOrigin::BottomEdge, SwipeDirection::Up, 150, -1_500)
    );
}

#[test]
fn left_edge_swipe_right_goes_back() {
    let mut gesture = gesture();
    gesture.begin(point(10, 250), 0, WIDTH, HEIGHT);
    gesture.update(point(150, 250), 50);
    assert_eq!(
        gesture.end(100),
        swipe(SwipeOrigin::LeftEdge, SwipeDirection::Right, 140, 1_400)
    );
}

#[test]
fn right_edge_swipe_left_is_recognized() {
    let mut gesture = gesture();
    gesture.begin(point(400, 250), 0, WIDTH, HEIGHT);
    gesture.update(point(260, 250), 50);
    assert_eq!(
        gesture.end(100),
        swipe(SwipeOrigin::RightEdge, SwipeDirection::Left, 140, -1_400)
    );
}
