//! Shell and app lifecycle failures.

use core::fmt;

use super::AppId;

/// An app lifecycle failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppError {
    /// The app rejected the request with a stable code.
    Failed(u16),
    /// The app is not in a state that accepts the request.
    InvalidState,
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Failed(code) => write!(formatter, "app failed with code {code}"),
            Self::InvalidState => formatter.write_str("app is in an invalid state"),
        }
    }
}

/// A shell navigation or input failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellError {
    /// The route stack is full.
    RouteFull,
    /// The fixed action queue is full.
    ActionQueueFull,
    /// An app kept queueing actions until the drain limit.
    ActionStorm,
    /// A modal layer blocks the request.
    Blocked,
    /// The route stack has no active app.
    NoActiveApp,
    /// No app is registered for the id.
    UnknownApp(AppId),
    /// An app returned an error.
    App(AppError),
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RouteFull => formatter.write_str("route stack full"),
            Self::ActionQueueFull => formatter.write_str("action queue full"),
            Self::ActionStorm => formatter.write_str("app action storm"),
            Self::Blocked => formatter.write_str("blocked by a modal layer"),
            Self::NoActiveApp => formatter.write_str("no active app"),
            Self::UnknownApp(id) => write!(formatter, "unknown app {}", id.value()),
            Self::App(error) => write!(formatter, "app error: {error}"),
        }
    }
}

impl From<AppError> for ShellError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}
