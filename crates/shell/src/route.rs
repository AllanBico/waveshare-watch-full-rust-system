//! The fixed app route stack.

use super::{AppId, ShellError};

/// Maximum routes retained by the shell.
pub const ROUTE_CAPACITY: usize = 8;

/// A fixed-capacity route stack with a pinned home route.
pub struct RouteStack {
    apps: [AppId; ROUTE_CAPACITY],
    len: usize,
}

impl RouteStack {
    /// Creates a stack containing only `home`.
    pub const fn new(home: AppId) -> Self {
        Self {
            apps: [home; ROUTE_CAPACITY],
            len: 1,
        }
    }

    /// The pinned home app.
    pub const fn home(&self) -> AppId {
        self.apps[0]
    }

    /// The active app.
    pub fn top(&self) -> Option<AppId> {
        self.apps.get(self.len.checked_sub(1)?).copied()
    }

    /// The number of routes, including home.
    pub const fn depth(&self) -> usize {
        self.len
    }

    /// Whether another route fits.
    pub const fn can_push(&self) -> bool {
        self.len < ROUTE_CAPACITY
    }

    /// The routes from home to the active app.
    pub fn routes(&self) -> &[AppId] {
        &self.apps[..self.len]
    }

    /// Whether `app` is anywhere on the stack.
    pub fn contains(&self, app: AppId) -> bool {
        self.routes().contains(&app)
    }

    /// Pushes an app route.
    pub fn push(&mut self, app: AppId) -> Result<(), ShellError> {
        let Some(slot) = self.apps.get_mut(self.len) else {
            return Err(ShellError::RouteFull);
        };
        *slot = app;
        self.len += 1;
        Ok(())
    }

    /// Pops the active route. The home route is never popped.
    pub fn pop(&mut self) -> Option<AppId> {
        if self.len <= 1 {
            return None;
        }
        self.len -= 1;
        self.apps.get(self.len).copied()
    }

    /// Pops every route above home.
    pub fn clear_to_home(&mut self) -> usize {
        let removed = self.len.saturating_sub(1);
        self.len = 1;
        removed
    }
}
