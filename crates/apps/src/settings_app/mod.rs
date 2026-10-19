//! The persisted settings screen.
//!
//! Watch settings expose only the values the firmware actually applies. The
//! app keeps a snapshot, applies edits locally for immediate feedback, and
//! queues one intent per change for the firmware-owned store.

mod app;
mod rows;

#[cfg(test)]
mod tests;

pub use app::SettingsApp;
