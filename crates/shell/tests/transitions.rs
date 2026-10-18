mod support;

use common::{Mode, Region};
use shell::{Action, LayerKind, Shell};
use support::{Log, Registry, HOME};

fn new_shell(log: Log) -> Shell<Registry> {
    Shell::new(
        Registry::new(log),
        HOME,
        Mode::Watch,
        Region::full(410, 502),
    )
}

#[test]
fn watch_overlays_snap_without_a_transition() {
    let mut shell = new_shell(Log::default());
    shell.start().unwrap();
    shell.dispatch(Action::OpenAppDrawer).unwrap();
    assert!(shell.transition().is_none());
    assert!(shell.layers().is_open(LayerKind::AppDrawer));

    shell.dispatch(Action::Back).unwrap();
    assert!(!shell.layers().is_open(LayerKind::AppDrawer));
}

#[test]
fn desk_overlays_animate_open_and_close() {
    let mut shell = Shell::new(
        Registry::new(Log::default()),
        HOME,
        Mode::Desk,
        Region::full(410, 502),
    );
    shell.start().unwrap();
    shell.dispatch(Action::OpenAppDrawer).unwrap();
    let transition = shell.transition().expect("desk opens with animation");
    assert!(transition.is_opening());
    assert_eq!(shell.transition_progress(150), Some(50));

    shell.tick(150).unwrap();
    assert!(shell.layers().is_open(LayerKind::AppDrawer));
    shell.tick(300).unwrap();
    assert!(shell.transition().is_none());
    assert!(shell.layers().is_open(LayerKind::AppDrawer));

    shell.dispatch(Action::Back).unwrap();
    let transition = shell.transition().expect("desk closes with animation");
    assert!(!transition.is_opening());
    assert!(shell.layers().is_open(LayerKind::AppDrawer));

    shell.tick(450).unwrap();
    assert_eq!(shell.transition_progress(450), Some(50));
    assert!(shell.layers().is_open(LayerKind::AppDrawer));
    shell.tick(600).unwrap();
    assert!(shell.transition().is_none());
    assert!(!shell.layers().is_open(LayerKind::AppDrawer));
}
