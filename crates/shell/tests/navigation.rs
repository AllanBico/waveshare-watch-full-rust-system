mod support;

use common::{Mode, Region, TouchPoint, TouchReport};
use shell::{
    Action, AppError, AppId, GestureOutcome, LayerKind, Shell, ShellError, ROUTE_CAPACITY,
};
use support::{Event, Log, Registry, HOME, OTHER};

fn new_shell(log: Log) -> Shell<Registry> {
    Shell::new(
        Registry::new(log),
        HOME,
        Mode::Watch,
        Region::full(410, 502),
    )
}

fn bottom_swipe(shell: &mut Shell<Registry>) -> GestureOutcome {
    let start = TouchReport::new(&[TouchPoint::new(1, 200, 500)]);
    let moved = TouchReport::new(&[TouchPoint::new(1, 200, 350)]);
    assert_eq!(shell.handle_touch(&start, 0).unwrap(), GestureOutcome::None);
    assert_eq!(
        shell.handle_touch(&moved, 50).unwrap(),
        GestureOutcome::None
    );
    shell.handle_touch(&TouchReport::empty(), 100).unwrap()
}

#[test]
fn back_order_is_dialog_overlay_route_home() {
    let log = Log::default();
    let mut shell = new_shell(log.clone());
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(OTHER)).unwrap();
    shell.dispatch(Action::OpenAppDrawer).unwrap();
    shell.dispatch(Action::OpenDialog(3)).unwrap();

    shell.dispatch(Action::Back).unwrap();
    assert!(!shell.layers().is_open(LayerKind::Dialog));
    assert!(shell.layers().is_open(LayerKind::AppDrawer));
    assert_eq!(shell.routes().depth(), 2);

    shell.dispatch(Action::Back).unwrap();
    assert!(!shell.layers().is_open(LayerKind::AppDrawer));
    assert_eq!(shell.routes().depth(), 2);

    shell.dispatch(Action::Back).unwrap();
    assert_eq!(shell.routes().depth(), 1);
    assert_eq!(shell.active_app(), Some(HOME));

    shell.dispatch(Action::Back).unwrap();
    assert_eq!(shell.routes().depth(), 1);

    assert_eq!(
        log.borrow().as_slice(),
        &[
            Event::Enter(HOME),
            Event::Exit(HOME),
            Event::Enter(OTHER),
            Event::Exit(OTHER),
            Event::Enter(HOME),
        ]
    );
}

#[test]
fn drawer_and_quick_settings_are_exclusive() {
    let mut shell = new_shell(Log::default());
    shell.start().unwrap();
    shell.dispatch(Action::OpenAppDrawer).unwrap();
    shell.dispatch(Action::OpenQuickSettings).unwrap();
    assert!(!shell.layers().is_open(LayerKind::AppDrawer));
    assert!(shell.layers().is_open(LayerKind::QuickSettings));

    shell.dispatch(Action::OpenAppDrawer).unwrap();
    assert!(shell.layers().is_open(LayerKind::AppDrawer));
    assert!(!shell.layers().is_open(LayerKind::QuickSettings));

    shell.dispatch(Action::OpenDialog(1)).unwrap();
    assert_eq!(
        shell.dispatch(Action::OpenQuickSettings),
        Err(ShellError::Blocked)
    );
    shell.dispatch(Action::Back).unwrap();
    assert!(!shell.layers().is_open(LayerKind::Dialog));
    assert!(shell.layers().is_open(LayerKind::AppDrawer));
}

#[test]
fn home_resets_routes_and_overlays_but_keeps_toasts() {
    let log = Log::default();
    let mut shell = new_shell(log.clone());
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(OTHER)).unwrap();
    shell.dispatch(Action::OpenAppDrawer).unwrap();
    shell.dispatch(Action::OpenDialog(2)).unwrap();
    shell
        .dispatch(Action::ShowToast {
            id: 4,
            duration_ms: 500,
        })
        .unwrap();

    shell.dispatch(Action::Home).unwrap();
    assert_eq!(shell.routes().depth(), 1);
    assert_eq!(shell.active_app(), Some(HOME));
    assert!(!shell.layers().is_open(LayerKind::AppDrawer));
    assert!(!shell.layers().is_open(LayerKind::Dialog));
    assert!(shell.layers().is_open(LayerKind::Toast));

    let before = log.borrow().len();
    shell.dispatch(Action::Home).unwrap();
    assert_eq!(log.borrow().len(), before);
}

#[test]
fn modal_blocks_actions_and_edge_gestures() {
    let log = Log::default();
    let mut shell = new_shell(log.clone());
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(OTHER)).unwrap();
    shell.dispatch(Action::OpenDialog(1)).unwrap();

    assert_eq!(shell.dispatch(Action::App(9)), Err(ShellError::Blocked));
    assert_eq!(shell.input_layer(), LayerKind::Dialog);
    assert!(shell.is_app_input_blocked());
    assert!(!log.borrow().contains(&Event::Action(OTHER, 9)));

    assert_eq!(bottom_swipe(&mut shell), GestureOutcome::None);
    assert!(!shell.layers().is_open(LayerKind::AppDrawer));

    shell.dispatch(Action::CloseDialog).unwrap();
    shell.dispatch(Action::App(9)).unwrap();
    assert!(log.borrow().contains(&Event::Action(OTHER, 9)));
}

#[test]
fn toast_never_takes_focus_or_blocks_app_input() {
    let log = Log::default();
    let mut shell = new_shell(log.clone());
    shell.start().unwrap();
    shell
        .dispatch(Action::ShowToast {
            id: 1,
            duration_ms: 100,
        })
        .unwrap();
    assert_eq!(shell.input_layer(), LayerKind::App);
    assert!(!shell.is_app_input_blocked());
    shell.dispatch(Action::App(5)).unwrap();
    assert!(log.borrow().contains(&Event::Action(HOME, 5)));

    shell.tick(99).unwrap();
    assert!(shell.layers().is_open(LayerKind::Toast));
    shell.tick(100).unwrap();
    assert!(!shell.layers().is_open(LayerKind::Toast));

    shell.dispatch(Action::OpenApp(OTHER)).unwrap();
    shell
        .dispatch(Action::ShowToast {
            id: 2,
            duration_ms: 500,
        })
        .unwrap();
    shell.dispatch(Action::Back).unwrap();
    assert_eq!(shell.active_app(), Some(HOME));
    assert!(shell.layers().is_open(LayerKind::Toast));

    shell.dispatch(Action::OpenDialog(3)).unwrap();
    assert_eq!(shell.input_layer(), LayerKind::Dialog);
}

#[test]
fn lifecycle_reaches_only_the_active_app() {
    let log = Log::default();
    let mut shell = new_shell(log.clone());
    shell.start().unwrap();
    shell.tick(10).unwrap();
    shell.dispatch(Action::OpenApp(OTHER)).unwrap();
    shell.tick(20).unwrap();
    shell.dispatch(Action::App(7)).unwrap();
    shell.dispatch(Action::Back).unwrap();
    shell.shutdown();

    assert_eq!(
        log.borrow().as_slice(),
        &[
            Event::Enter(HOME),
            Event::Tick(HOME),
            Event::Exit(HOME),
            Event::Enter(OTHER),
            Event::Tick(OTHER),
            Event::Action(OTHER, 7),
            Event::Exit(OTHER),
            Event::Enter(HOME),
            Event::Exit(HOME),
        ]
    );
}

#[test]
fn mode_switch_reenters_the_active_app_and_clears_overlays() {
    let log = Log::default();
    let mut shell = new_shell(log.clone());
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(OTHER)).unwrap();
    shell.dispatch(Action::OpenDialog(1)).unwrap();

    shell.set_mode(Mode::Desk).unwrap();
    assert_eq!(shell.mode(), Mode::Desk);
    assert!(shell.layers().is_empty());
    assert_eq!(
        log.borrow().as_slice(),
        &[
            Event::Enter(HOME),
            Event::Exit(HOME),
            Event::Enter(OTHER),
            Event::Exit(OTHER),
            Event::Enter(OTHER),
        ]
    );
}

#[test]
fn unknown_app_is_rejected_without_route_change() {
    let mut shell = new_shell(Log::default());
    shell.start().unwrap();
    let unknown = AppId::new(99);
    assert_eq!(
        shell.dispatch(Action::OpenApp(unknown)),
        Err(ShellError::UnknownApp(unknown))
    );
    assert_eq!(shell.routes().depth(), 1);
    assert_eq!(shell.active_app(), Some(HOME));
}

#[test]
fn failed_enter_rolls_back_to_the_previous_route() {
    let log = Log::default();
    let mut shell = Shell::new(
        Registry::with_failing_other(log.clone()),
        HOME,
        Mode::Watch,
        Region::full(410, 502),
    );
    shell.start().unwrap();
    assert_eq!(
        shell.dispatch(Action::OpenApp(OTHER)),
        Err(ShellError::App(AppError::Failed(7)))
    );
    assert_eq!(shell.routes().depth(), 1);
    assert_eq!(shell.active_app(), Some(HOME));
    assert_eq!(
        log.borrow().as_slice(),
        &[
            Event::Enter(HOME),
            Event::Exit(HOME),
            Event::Enter(OTHER),
            Event::Enter(HOME),
        ]
    );
}

#[test]
fn route_stack_rejects_overflow_without_side_effects() {
    let log = Log::default();
    let mut shell = new_shell(log.clone());
    shell.start().unwrap();
    for index in 0..(ROUTE_CAPACITY - 1) {
        let app = if index % 2 == 0 { OTHER } else { HOME };
        shell.dispatch(Action::OpenApp(app)).unwrap();
    }
    let before = log.borrow().len();
    assert_eq!(
        shell.dispatch(Action::OpenApp(HOME)),
        Err(ShellError::RouteFull)
    );
    assert_eq!(log.borrow().len(), before);
}

#[test]
fn app_action_storm_is_bounded() {
    let log = Log::default();
    let mut shell = Shell::new(
        Registry::with_storming_other(log),
        HOME,
        Mode::Watch,
        Region::full(410, 502),
    );
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(OTHER)).unwrap();
    assert_eq!(shell.dispatch(Action::App(1)), Err(ShellError::ActionStorm));
}

#[test]
fn content_swipes_dismiss_open_overlays() {
    let mut shell = new_shell(Log::default());
    shell.start().unwrap();

    shell.dispatch(Action::OpenAppDrawer).unwrap();
    let start = TouchReport::new(&[TouchPoint::new(1, 200, 250)]);
    let moved = TouchReport::new(&[TouchPoint::new(1, 200, 380)]);
    shell.handle_touch(&start, 0).unwrap();
    shell.handle_touch(&moved, 50).unwrap();
    shell.handle_touch(&TouchReport::empty(), 100).unwrap();
    assert!(!shell.layers().is_open(LayerKind::AppDrawer));

    shell.dispatch(Action::OpenQuickSettings).unwrap();
    let start = TouchReport::new(&[TouchPoint::new(1, 200, 300)]);
    let moved = TouchReport::new(&[TouchPoint::new(1, 200, 150)]);
    shell.handle_touch(&start, 0).unwrap();
    shell.handle_touch(&moved, 50).unwrap();
    shell.handle_touch(&TouchReport::empty(), 100).unwrap();
    assert!(!shell.layers().is_open(LayerKind::QuickSettings));
}
