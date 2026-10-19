//! App-level Back and content-swipe routing tests.

use std::cell::RefCell;
use std::rc::Rc;

use common::{Mode, Region, TouchPoint, TouchReport};
use shell::{Action, App, AppContext, AppError, AppId, AppRegistry, Shell, Swipe, SwipeOrigin};

const HOME: AppId = AppId::new(1);
const LOGS: AppId = AppId::new(2);

#[derive(Default)]
struct Seen {
    swipes: Vec<SwipeOrigin>,
    backs: u8,
}

struct Handler {
    handles_back: bool,
    seen: Rc<RefCell<Seen>>,
}

impl App for Handler {
    fn swipe(&mut self, swipe: Swipe, _context: &mut AppContext<'_>) -> Result<bool, AppError> {
        self.seen.borrow_mut().swipes.push(swipe.origin);
        Ok(true)
    }

    fn back(&mut self, _context: &mut AppContext<'_>) -> Result<bool, AppError> {
        self.seen.borrow_mut().backs += 1;
        Ok(self.handles_back)
    }
}

struct Registry {
    home: Handler,
    logs: Handler,
}

impl AppRegistry for Registry {
    fn with_app<R>(&mut self, id: AppId, f: impl FnOnce(&mut dyn App) -> R) -> Option<R> {
        match id {
            HOME => Some(f(&mut self.home)),
            LOGS => Some(f(&mut self.logs)),
            _ => None,
        }
    }
}

fn shell(seen: Rc<RefCell<Seen>>) -> Shell<Registry> {
    Shell::new(
        Registry {
            home: Handler {
                handles_back: false,
                seen: seen.clone(),
            },
            logs: Handler {
                handles_back: true,
                seen,
            },
        },
        HOME,
        Mode::Watch,
        Region::full(410, 502),
    )
}

fn content_swipe(shell: &mut Shell<Registry>, from_y: u16, to_y: u16) {
    let start = TouchReport::new(&[TouchPoint::new(1, 200, from_y)]);
    let moved = TouchReport::new(&[TouchPoint::new(1, 200, to_y)]);
    shell.handle_touch(&start, 0).unwrap();
    shell.handle_touch(&moved, 50).unwrap();
    shell.handle_touch(&TouchReport::empty(), 100).unwrap();
}

#[test]
fn app_back_intercepts_before_route_pop() {
    let seen = Rc::new(RefCell::new(Seen::default()));
    let mut shell = shell(seen.clone());
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(LOGS)).unwrap();
    assert_eq!(shell.routes().depth(), 2);

    shell.dispatch(Action::Back).unwrap();
    assert_eq!(shell.routes().depth(), 2);
    assert_eq!(seen.borrow().backs, 1);
}

#[test]
fn content_swipes_reach_the_active_app() {
    let seen = Rc::new(RefCell::new(Seen::default()));
    let mut shell = shell(seen.clone());
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(LOGS)).unwrap();

    content_swipe(&mut shell, 300, 150);
    assert_eq!(seen.borrow().swipes, vec![SwipeOrigin::Content]);
    assert!(!shell.layers().is_open(shell::LayerKind::QuickSettings));
}

#[test]
fn edge_swipes_stay_global() {
    let seen = Rc::new(RefCell::new(Seen::default()));
    let mut shell = shell(seen.clone());
    shell.start().unwrap();
    shell.dispatch(Action::OpenApp(LOGS)).unwrap();

    content_swipe(&mut shell, 5, 140);
    assert!(seen.borrow().swipes.is_empty());
    assert!(shell.layers().is_open(shell::LayerKind::QuickSettings));
}
