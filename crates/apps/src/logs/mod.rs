//! The persistent log viewer.
//!
//! The firmware loads the newest records into a fixed snapshot; this app only
//! filters, pages, and formats them. It never touches flash.

mod app;
mod format;
mod model;
mod view;

#[cfg(test)]
mod tests;

pub use app::LogsApp;
pub use model::LogCategory;
