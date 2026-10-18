//! App shell: lifecycle, navigation, layers, gestures, and status.
//!
//! The shell owns global navigation, overlay layers, focus routing, and the
//! status model. Apps own their state and depend on the shell, never the other
//! way around. The firmware implements [`AppRegistry`] and wires concrete apps
//! into [`Shell`].

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod action;
mod app;
mod context;
mod error;
mod gesture;
mod input;
mod layer;
mod route;
mod shell;
mod status;
mod transition;

pub use action::{Action, ActionQueue, ACTION_CAPACITY};
pub use app::{App, AppId, AppRegistry};
pub use context::AppContext;
pub use error::{AppError, ShellError};
pub use gesture::{
    EdgeGesture, GestureConfig, GestureOutcome, GesturePhase, Swipe, SwipeDirection, SwipeOrigin,
};
pub use input::InputRouter;
pub use layer::{Layer, LayerKind, LayerStack, LAYER_CAPACITY};
pub use route::{RouteStack, ROUTE_CAPACITY};
pub use shell::{Shell, MAX_ACTIONS_PER_DISPATCH};
pub use status::StatusModel;
pub use transition::LayerTransition;
