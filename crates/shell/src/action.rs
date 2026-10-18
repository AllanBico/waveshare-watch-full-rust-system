//! Abstract navigation actions and the fixed action queue.

use super::{AppId, ShellError};

/// Maximum actions an app may queue between drains.
pub const ACTION_CAPACITY: usize = 8;

/// An abstract request interpreted by the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Close the top dialog, overlay, or route.
    Back,
    /// Return to the pinned home app.
    Home,
    /// Push an app route.
    OpenApp(AppId),
    /// Open the app drawer from the bottom edge.
    OpenAppDrawer,
    /// Close the app drawer.
    CloseAppDrawer,
    /// Open quick settings from the top edge.
    OpenQuickSettings,
    /// Close quick settings.
    CloseQuickSettings,
    /// Open or replace the dialog layer.
    OpenDialog(u16),
    /// Close the dialog layer.
    CloseDialog,
    /// Show or replace the toast layer.
    ShowToast {
        /// Toast content identifier.
        id: u16,
        /// Time until the toast expires.
        duration_ms: u32,
    },
    /// Dismiss the toast layer.
    DismissToast,
    /// A request addressed to the active app.
    App(u16),
}

/// A fixed-capacity FIFO of actions queued by apps.
pub struct ActionQueue {
    actions: [Option<Action>; ACTION_CAPACITY],
    len: usize,
}

impl ActionQueue {
    /// Creates an empty queue.
    pub const fn new() -> Self {
        Self {
            actions: [None; ACTION_CAPACITY],
            len: 0,
        }
    }

    /// The queue capacity.
    pub const fn capacity() -> usize {
        ACTION_CAPACITY
    }

    /// The number of queued actions.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the queue is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends one action.
    pub fn push(&mut self, action: Action) -> Result<(), ShellError> {
        let Some(slot) = self.actions.get_mut(self.len) else {
            return Err(ShellError::ActionQueueFull);
        };
        *slot = Some(action);
        self.len += 1;
        Ok(())
    }

    /// Removes and returns the oldest action.
    pub fn pop(&mut self) -> Option<Action> {
        if self.len == 0 {
            return None;
        }
        let action = self.actions[0].take();
        for index in 1..self.len {
            self.actions[index - 1] = self.actions[index].take();
        }
        self.len -= 1;
        action
    }

    /// Removes every queued action.
    pub fn clear(&mut self) {
        self.actions = [None; ACTION_CAPACITY];
        self.len = 0;
    }
}

impl Default for ActionQueue {
    fn default() -> Self {
        Self::new()
    }
}
