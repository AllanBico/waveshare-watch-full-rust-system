//! The app trait, app identifiers, and the firmware-owned registry.

use core::fmt;

use ui::{Damage, Surface, Theme};

use super::{Action, AppContext, AppError, Swipe};

/// A stable identifier for one app implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AppId(u8);

impl AppId {
    /// Creates an app identifier.
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    /// The raw identifier.
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl fmt::Display for AppId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// One app's lifecycle, driven by the shell.
///
/// Apps own their state. They never navigate globally and never touch
/// hardware; navigation is expressed by queueing [`Action`]s through
/// [`AppContext`].
pub trait App {
    /// Called when the app becomes the active route.
    fn enter(&mut self, context: &mut AppContext<'_>) -> Result<(), AppError> {
        let _ = context;
        Ok(())
    }

    /// Called when the app stops being the active route.
    fn exit(&mut self, context: &mut AppContext<'_>) {
        let _ = context;
    }

    /// Called once per frame while the app is active and unblocked.
    fn tick(&mut self, context: &mut AppContext<'_>) {
        let _ = context;
    }

    /// Handles one action addressed to this app.
    fn action(&mut self, action: Action, context: &mut AppContext<'_>) -> Result<(), AppError> {
        let _ = (action, context);
        Ok(())
    }

    /// Handles a tap at screen coordinates.
    fn tap(&mut self, x: u16, y: u16, context: &mut AppContext<'_>) -> Result<(), AppError> {
        let _ = (x, y, context);
        Ok(())
    }

    /// Handles a content swipe, returning whether the app consumed it.
    fn swipe(&mut self, swipe: Swipe, context: &mut AppContext<'_>) -> Result<bool, AppError> {
        let _ = (swipe, context);
        Ok(false)
    }

    /// Intercepts Back, returning whether the app handled it.
    fn back(&mut self, context: &mut AppContext<'_>) -> Result<bool, AppError> {
        let _ = context;
        Ok(false)
    }

    /// Marks the app's next frame as a full redraw.
    fn invalidate(&mut self) {}

    /// Builds, diffs, and paints one frame, returning the damage to flush.
    ///
    /// Apps own their transient render state. The shell supplies the current
    /// context, the compiled-in theme, and the surface the firmware owns.
    fn render(
        &mut self,
        context: &AppContext<'_>,
        theme: &Theme,
        surface: &mut dyn Surface,
    ) -> Result<Damage, AppError> {
        let _ = (context, theme, surface);
        Ok(Damage::new())
    }
}

/// Resolves app identifiers to app implementations.
///
/// The firmware implements this trait so the shell never depends on concrete
/// apps. The callback borrows exactly one app, which keeps apps isolated from
/// each other.
pub trait AppRegistry {
    /// Runs `f` with the app registered under `id`, if any.
    fn with_app<R>(&mut self, id: AppId, f: impl FnOnce(&mut dyn App) -> R) -> Option<R>;

    /// Runs `f` with the overlay registered for `kind`, if any.
    fn with_layer<R>(
        &mut self,
        kind: crate::LayerKind,
        f: impl FnOnce(&mut dyn App) -> R,
    ) -> Option<R> {
        let _ = (kind, f);
        None
    }
}
