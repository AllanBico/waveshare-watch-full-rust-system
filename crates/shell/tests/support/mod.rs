//! Shared fake apps for shell integration tests.

#![allow(dead_code)]

use std::cell::RefCell;
use std::rc::Rc;

use common::Percent;
use shell::{Action, App, AppContext, AppError, AppId, AppRegistry};

pub const HOME: AppId = AppId::new(1);
pub const OTHER: AppId = AppId::new(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Enter(AppId),
    Exit(AppId),
    Tick(AppId),
    Action(AppId, u16),
    Invalidate(AppId),
}

pub type Log = Rc<RefCell<Vec<Event>>>;

pub struct LoggedApp {
    id: AppId,
    log: Log,
    fail_enter: bool,
    storm: bool,
    battery: Option<(Percent, bool)>,
}

impl LoggedApp {
    pub fn new(id: AppId, log: Log) -> Self {
        Self {
            id,
            log,
            fail_enter: false,
            storm: false,
            battery: None,
        }
    }

    pub fn with_failing_enter(id: AppId, log: Log) -> Self {
        Self {
            fail_enter: true,
            ..Self::new(id, log)
        }
    }

    pub fn with_storm(id: AppId, log: Log) -> Self {
        Self {
            storm: true,
            ..Self::new(id, log)
        }
    }

    pub fn with_battery(id: AppId, log: Log, percent: Percent, is_charging: bool) -> Self {
        Self {
            battery: Some((percent, is_charging)),
            ..Self::new(id, log)
        }
    }

    fn record(&self, event: Event) {
        self.log.borrow_mut().push(event);
    }
}

impl App for LoggedApp {
    fn enter(&mut self, context: &mut AppContext<'_>) -> Result<(), AppError> {
        self.record(Event::Enter(self.id));
        if let Some((percent, is_charging)) = self.battery {
            context.publish_battery(percent, is_charging);
        }
        if self.fail_enter {
            Err(AppError::Failed(7))
        } else {
            Ok(())
        }
    }

    fn exit(&mut self, _context: &mut AppContext<'_>) {
        self.record(Event::Exit(self.id));
    }

    fn tick(&mut self, _context: &mut AppContext<'_>) {
        self.record(Event::Tick(self.id));
    }

    fn invalidate(&mut self) {
        self.record(Event::Invalidate(self.id));
    }

    fn action(&mut self, action: Action, context: &mut AppContext<'_>) -> Result<(), AppError> {
        if let Action::App(code) = action {
            self.record(Event::Action(self.id, code));
            if self.storm {
                let _ = context.push_action(Action::App(code));
            }
        }
        Ok(())
    }
}

pub struct Registry {
    home: LoggedApp,
    other: LoggedApp,
}

impl Registry {
    pub fn new(log: Log) -> Self {
        Self {
            home: LoggedApp::new(HOME, log.clone()),
            other: LoggedApp::new(OTHER, log),
        }
    }

    pub fn with_failing_other(log: Log) -> Self {
        Self {
            home: LoggedApp::new(HOME, log.clone()),
            other: LoggedApp::with_failing_enter(OTHER, log),
        }
    }

    pub fn with_storming_other(log: Log) -> Self {
        Self {
            home: LoggedApp::new(HOME, log.clone()),
            other: LoggedApp::with_storm(OTHER, log),
        }
    }

    pub fn with_battery(log: Log, percent: Percent) -> Self {
        Self {
            home: LoggedApp::with_battery(HOME, log.clone(), percent, true),
            other: LoggedApp::new(OTHER, log),
        }
    }
}

impl AppRegistry for Registry {
    fn with_app<R>(&mut self, id: AppId, f: impl FnOnce(&mut dyn App) -> R) -> Option<R> {
        match id {
            HOME => Some(f(&mut self.home)),
            OTHER => Some(f(&mut self.other)),
            _ => None,
        }
    }
}
