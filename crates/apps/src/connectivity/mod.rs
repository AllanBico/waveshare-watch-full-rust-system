//! The desk-only network status screen.
//!
//! The app only renders a firmware-supplied snapshot and queues start/stop
//! intents. It never touches the radio.

mod app;
mod format;

pub use app::{ConnectivityApp, Intent as ConnectivityIntent};
