//! Action interpretation, routes, and the fixed back order.

use crate::{Action, AppId, AppRegistry, LayerKind, ShellError};

use super::{Shell, MAX_ACTIONS_PER_DISPATCH};

impl<R: AppRegistry> Shell<R> {
    /// Goes back one step using the fixed back order.
    pub fn back(&mut self) -> Result<(), ShellError> {
        if self.layers.is_open(LayerKind::Dialog) {
            self.close_layer(LayerKind::Dialog);
            return Ok(());
        }
        if self.layers.is_open(LayerKind::QuickSettings) {
            self.close_layer(LayerKind::QuickSettings);
            return Ok(());
        }
        if self.layers.is_open(LayerKind::AppDrawer) {
            self.close_layer(LayerKind::AppDrawer);
            return Ok(());
        }
        self.pop_route()
    }

    pub(super) fn apply_action(&mut self, action: Action) -> Result<(), ShellError> {
        match action {
            Action::Back => self.back(),
            Action::Home => self.go_home(),
            Action::OpenApp(id) => self.open_app(id),
            Action::OpenAppDrawer => self.open_app_drawer(),
            Action::CloseAppDrawer => {
                self.close_layer(LayerKind::AppDrawer);
                Ok(())
            }
            Action::OpenQuickSettings => self.open_quick_settings(),
            Action::CloseQuickSettings => {
                self.close_layer(LayerKind::QuickSettings);
                Ok(())
            }
            Action::OpenDialog(id) => {
                self.input.cancel();
                self.open_layer(LayerKind::Dialog, id);
                Ok(())
            }
            Action::CloseDialog => {
                self.close_layer(LayerKind::Dialog);
                Ok(())
            }
            Action::ShowToast { id, duration_ms } => self.show_toast(id, duration_ms),
            Action::DismissToast => {
                self.layers.close(LayerKind::Toast);
                self.toast_deadline = None;
                Ok(())
            }
            Action::App(code) => self.deliver_app_action(code),
        }
    }

    pub(super) fn apply_swipe(&mut self, swipe: crate::Swipe) -> Result<(), ShellError> {
        use crate::{SwipeDirection, SwipeOrigin};
        match (swipe.origin, swipe.direction) {
            (SwipeOrigin::BottomEdge, SwipeDirection::Up) => self.dispatch(Action::OpenAppDrawer),
            (SwipeOrigin::BottomEdge, SwipeDirection::Down) => {
                self.dispatch(Action::CloseAppDrawer)
            }
            (SwipeOrigin::TopEdge, SwipeDirection::Down) => {
                self.dispatch(Action::OpenQuickSettings)
            }
            (SwipeOrigin::TopEdge, SwipeDirection::Up) => self.dispatch(Action::CloseQuickSettings),
            _ => Ok(()),
        }
    }

    fn open_app(&mut self, id: AppId) -> Result<(), ShellError> {
        if self.layers.is_modal() {
            return Err(ShellError::Blocked);
        }
        self.layers.close(LayerKind::AppDrawer);
        self.layers.close(LayerKind::QuickSettings);
        self.transition = None;
        if self.active_app() == Some(id) {
            return Ok(());
        }
        if !self.routes.can_push() {
            return Err(ShellError::RouteFull);
        }
        let previous = self.active_app();
        self.routes.push(id)?;
        if self.is_started {
            if let Some(previous) = previous {
                self.call_exit(previous);
            }
        }
        if let Err(error) = self.enter_route(id) {
            let _ = self.routes.pop();
            if self.is_started {
                if let Some(previous) = previous {
                    let _ = self.call_enter(previous);
                }
            }
            return Err(error);
        }
        Ok(())
    }

    fn open_app_drawer(&mut self) -> Result<(), ShellError> {
        if self.layers.is_modal() {
            return Err(ShellError::Blocked);
        }
        self.layers.close(LayerKind::QuickSettings);
        self.open_layer(LayerKind::AppDrawer, 0);
        Ok(())
    }

    fn open_quick_settings(&mut self) -> Result<(), ShellError> {
        if self.layers.is_modal() {
            return Err(ShellError::Blocked);
        }
        self.layers.close(LayerKind::AppDrawer);
        self.open_layer(LayerKind::QuickSettings, 0);
        Ok(())
    }

    fn go_home(&mut self) -> Result<(), ShellError> {
        self.layers.clear_overlays();
        self.transition = None;
        let home = self.routes.home();
        if self.routes.depth() == 1 && self.active_app() == Some(home) {
            return Ok(());
        }
        let active = self.active_app();
        if self.is_started {
            if let Some(id) = active {
                self.call_exit(id);
            }
        }
        self.routes.clear_to_home();
        if self.is_started {
            self.call_enter(home)?;
        }
        Ok(())
    }

    fn pop_route(&mut self) -> Result<(), ShellError> {
        let Some(popped) = self.routes.pop() else {
            return Ok(());
        };
        if self.is_started {
            self.call_exit(popped);
        }
        let Some(next) = self.active_app() else {
            return Ok(());
        };
        if let Err(error) = self.enter_route(next) {
            if self.is_started {
                let _ = self.call_enter(popped);
            }
            let _ = self.routes.push(popped);
            return Err(error);
        }
        Ok(())
    }

    fn deliver_app_action(&mut self, code: u16) -> Result<(), ShellError> {
        if self.is_app_input_blocked() {
            return Err(ShellError::Blocked);
        }
        let id = self.active_app().ok_or(ShellError::NoActiveApp)?;
        self.call_app(id, |app, context| app.action(Action::App(code), context))??;
        Ok(())
    }

    pub(super) fn drain_actions(&mut self) -> Result<(), ShellError> {
        let mut processed = 0u8;
        while let Some(action) = self.actions.pop() {
            if let Err(error) = self.apply_action(action) {
                self.actions.clear();
                return Err(error);
            }
            processed = processed.saturating_add(1);
            if processed >= MAX_ACTIONS_PER_DISPATCH {
                self.actions.clear();
                return Err(ShellError::ActionStorm);
            }
        }
        Ok(())
    }
}
