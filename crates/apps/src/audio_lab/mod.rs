//! Desk-only Audio Lab: loopback, microphone, and speaker tuning.
//!
//! The app is pure. It emits [`voice::lab::LabIntent`]s for the firmware-owned
//! audio service and renders the [`voice::lab::LabEvent`]s it receives back.

mod app;
mod format;
mod view;

#[cfg(test)]
mod tests;

pub use app::{AudioLab, RenderError};
