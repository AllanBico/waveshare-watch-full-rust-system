//! Power management.
//!
//! Owns all rails, sleep decisions, and the render policy. Single source of
//! truth for power state and mode transitions. Hardware-agnostic: it drives a
//! [`hal::Power`] and holds no register knowledge.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod budget;
mod error;
mod manager;
mod plan;

#[cfg(test)]
mod tests;

pub use budget::{Budget, BudgetStatus};
pub use error::PowerError;
pub use manager::PowerManager;
pub use plan::{PowerState, RailPlan};
