//! Shell orchestration: lifecycle, navigation, layers, and input.

mod layers;
mod navigation;

use common::{Mode, Percent, Region, RenderPolicy, Timestamp, TouchReport};

use super::{
    Action, ActionQueue, App, AppContext, AppId, AppRegistry, GestureConfig, GestureOutcome,
    InputRouter, LayerKind, LayerStack, LayerTransition, RouteStack, ShellError, StatusModel,
};

use crate::context::ContextInput;

/// Maximum actions drained after one dispatch or tick.
pub const MAX_ACTIONS_PER_DISPATCH: u8 = 8;

/// The app shell.
///
/// Owns navigation, overlays, status, and global gestures. Concrete apps live
/// behind the firmware-implemented [`AppRegistry`].
pub struct Shell<R> {
    registry: R,
    routes: RouteStack,
    layers: LayerStack,
    input: InputRouter,
    status: StatusModel,
    actions: ActionQueue,
    mode: Mode,
    policy: RenderPolicy,
    now: Timestamp,
    uptime_ms: u64,
    viewport: Region,
    toast_deadline: Option<u64>,
    transition: Option<LayerTransition>,
    is_started: bool,
}

impl<R: AppRegistry> Shell<R> {
    /// Creates a shell with `home` pinned as the base route.
    pub fn new(registry: R, home: AppId, mode: Mode, viewport: Region) -> Self {
        Self {
            registry,
            routes: RouteStack::new(home),
            layers: LayerStack::new(),
            input: InputRouter::new(GestureConfig::default()),
            status: StatusModel::new(),
            actions: ActionQueue::new(),
            mode,
            policy: RenderPolicy::for_mode(mode),
            now: Timestamp::from_millis(0),
            uptime_ms: 0,
            viewport,
            toast_deadline: None,
            transition: None,
            is_started: false,
        }
    }

    /// The app registry.
    pub const fn registry(&self) -> &R {
        &self.registry
    }

    /// The app registry, mutably.
    pub fn registry_mut(&mut self) -> &mut R {
        &mut self.registry
    }

    /// The active runtime mode.
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The active render policy.
    pub const fn policy(&self) -> RenderPolicy {
        self.policy
    }

    /// The last wall-clock time.
    pub const fn now(&self) -> Timestamp {
        self.now
    }

    /// The current screen rectangle.
    pub const fn viewport(&self) -> Region {
        self.viewport
    }

    /// The shell-owned status model.
    pub const fn status(&self) -> &StatusModel {
        &self.status
    }

    /// The in-flight overlay transition, when the mode animates.
    pub const fn transition(&self) -> Option<LayerTransition> {
        self.transition
    }

    /// The in-flight transition progress in `0..=100`.
    pub fn transition_progress(&self, now_ms: u64) -> Option<u8> {
        self.transition
            .map(|transition| transition.progress(now_ms))
    }

    /// The app route stack.
    pub const fn routes(&self) -> &RouteStack {
        &self.routes
    }

    /// The overlay layer stack.
    pub const fn layers(&self) -> &LayerStack {
        &self.layers
    }

    /// The active app, when the shell has a route.
    pub fn active_app(&self) -> Option<AppId> {
        self.routes.top()
    }

    /// Whether the shell has entered its home app.
    pub const fn is_started(&self) -> bool {
        self.is_started
    }

    /// The layer that should receive a tap.
    pub fn input_layer(&self) -> LayerKind {
        self.layers
            .top_interactive()
            .map(|layer| layer.kind)
            .unwrap_or(LayerKind::App)
    }

    /// Whether an overlay or dialog blocks app input.
    pub fn is_app_input_blocked(&self) -> bool {
        self.layers.blocks_app_input()
    }

    /// Updates the viewport rectangle.
    pub fn set_viewport(&mut self, viewport: Region) {
        self.viewport = viewport;
    }

    /// Updates the shell clock.
    pub fn set_time(&mut self, now: Timestamp) {
        self.now = now;
        self.status.set_clock(now);
    }

    /// Updates the status battery values.
    pub fn set_battery(&mut self, percent: Percent, is_charging: bool) {
        self.status.set_battery(percent, is_charging);
    }

    /// Switches runtime mode and re-enters the active app.
    pub fn set_mode(&mut self, mode: Mode) -> Result<(), ShellError> {
        if self.mode == mode {
            return Ok(());
        }
        let active = self.active_app();
        if self.is_started {
            if let Some(id) = active {
                self.call_exit(id);
            }
        }
        self.layers.clear_all();
        self.toast_deadline = None;
        self.transition = None;
        self.actions.clear();
        self.mode = mode;
        self.policy = RenderPolicy::for_mode(mode);
        if self.is_started {
            if let Some(id) = active {
                self.call_enter(id)?;
            }
        }
        Ok(())
    }

    /// Enters the home app once.
    pub fn start(&mut self) -> Result<(), ShellError> {
        if self.is_started {
            return Ok(());
        }
        let id = self.active_app().ok_or(ShellError::NoActiveApp)?;
        self.call_enter(id)?;
        self.is_started = true;
        Ok(())
    }

    /// Leaves the active app and clears transient shell state.
    pub fn shutdown(&mut self) {
        if !self.is_started {
            return;
        }
        if let Some(id) = self.active_app() {
            self.call_exit(id);
        }
        self.is_started = false;
        self.layers.clear_all();
        self.transition = None;
        self.toast_deadline = None;
        self.actions.clear();
    }

    /// Handles global edge gestures for one touch report.
    pub fn handle_touch(
        &mut self,
        report: &TouchReport,
        now_ms: u64,
    ) -> Result<GestureOutcome, ShellError> {
        if self.layers.is_modal() {
            self.input.cancel();
            return Ok(GestureOutcome::None);
        }
        self.uptime_ms = now_ms;
        let outcome = self.input.handle(report, now_ms, self.viewport.height);
        if let GestureOutcome::Swipe(swipe) = outcome {
            self.apply_swipe(swipe)?;
        }
        Ok(outcome)
    }

    /// Advances the shell and the active app.
    pub fn tick(&mut self, now_ms: u64) -> Result<(), ShellError> {
        self.uptime_ms = now_ms;
        self.expire_toast(now_ms);
        self.advance_transition(now_ms);
        if self.is_started && !self.is_app_input_blocked() {
            if let Some(id) = self.active_app() {
                self.call_app(id, |app, context| app.tick(context))?;
            }
        }
        self.drain_actions()
    }

    /// Dispatches one action and drains actions queued by apps.
    pub fn dispatch(&mut self, action: Action) -> Result<(), ShellError> {
        if let Err(error) = self.apply_action(action) {
            self.actions.clear();
            return Err(error);
        }
        self.drain_actions()
    }

    fn enter_route(&mut self, id: AppId) -> Result<(), ShellError> {
        if self.is_started {
            self.call_enter(id)
        } else {
            Ok(())
        }
    }

    fn call_enter(&mut self, id: AppId) -> Result<(), ShellError> {
        self.call_app(id, |app, context| app.enter(context))??;
        Ok(())
    }

    fn call_exit(&mut self, id: AppId) {
        let _ = self.call_app(id, |app, context| app.exit(context));
    }

    fn call_app<R2>(
        &mut self,
        id: AppId,
        f: impl FnOnce(&mut dyn App, &mut AppContext<'_>) -> R2,
    ) -> Result<R2, ShellError> {
        let Self {
            registry,
            mode,
            policy,
            now,
            uptime_ms,
            viewport,
            status,
            actions,
            ..
        } = self;
        let mut context = AppContext::new(ContextInput {
            mode: *mode,
            policy: *policy,
            now: *now,
            uptime_ms: *uptime_ms,
            viewport: *viewport,
            app: id,
            status,
            actions,
        });
        registry
            .with_app(id, |app| f(app, &mut context))
            .ok_or(ShellError::UnknownApp(id))
    }
}
