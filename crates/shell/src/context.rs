//! The per-call handle apps use to reach shell-owned services.

use common::{Mode, Percent, Region, RenderPolicy, Timestamp};
use ui::Theme;

use super::{Action, ActionQueue, AppId, ShellError, StatusModel};

/// Shared services exposed to one app call.
///
/// The shell constructs a context for each lifecycle call. Apps may read the
/// runtime state, publish status values, and queue abstract actions. They
/// cannot reach other apps, the route stack, or the layer stack directly.
pub struct AppContext<'a> {
    mode: Mode,
    policy: RenderPolicy,
    theme: &'static Theme,
    now: Timestamp,
    uptime_ms: u64,
    viewport: Region,
    app: AppId,
    status: &'a mut StatusModel,
    actions: &'a mut ActionQueue,
}

/// The shell state needed to build one [`AppContext`].
pub(crate) struct ContextInput<'a> {
    /// Runtime mode.
    pub mode: Mode,
    /// Render policy.
    pub policy: RenderPolicy,
    /// Wall-clock time.
    pub now: Timestamp,
    /// Monotonic uptime.
    pub uptime_ms: u64,
    /// Screen rectangle.
    pub viewport: Region,
    /// Receiving app.
    pub app: AppId,
    /// Shell status model.
    pub status: &'a mut StatusModel,
    /// Shell action queue.
    pub actions: &'a mut ActionQueue,
}

impl<'a> AppContext<'a> {
    pub(crate) fn new(input: ContextInput<'a>) -> Self {
        Self {
            mode: input.mode,
            policy: input.policy,
            theme: Theme::for_pixel_mode(input.policy.pixel),
            now: input.now,
            uptime_ms: input.uptime_ms,
            viewport: input.viewport,
            app: input.app,
            status: input.status,
            actions: input.actions,
        }
    }

    /// The current runtime mode.
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The current render policy.
    pub const fn policy(&self) -> RenderPolicy {
        self.policy
    }

    /// The compiled-in theme for the current pixel mode.
    pub const fn theme(&self) -> &'static Theme {
        self.theme
    }

    /// The last wall-clock time supplied by the firmware.
    pub const fn now(&self) -> Timestamp {
        self.now
    }

    /// Monotonic uptime supplied by the firmware.
    pub const fn uptime_ms(&self) -> u64 {
        self.uptime_ms
    }

    /// The current screen rectangle.
    pub const fn viewport(&self) -> Region {
        self.viewport
    }

    /// The id of the app receiving this context.
    pub const fn app(&self) -> AppId {
        self.app
    }

    /// The shell-owned status model.
    pub fn status(&self) -> &StatusModel {
        self.status
    }

    /// Publishes a battery level and charging state.
    pub fn publish_battery(&mut self, percent: Percent, is_charging: bool) {
        self.status.set_battery(percent, is_charging);
    }

    /// Queues an abstract action for the shell.
    pub fn push_action(&mut self, action: Action) -> Result<(), ShellError> {
        self.actions.push(action)
    }
}
