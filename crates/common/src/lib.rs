//! Shared vocabulary for the whole system.
//!
//! Owns the types every other crate depends on: [`Mode`], [`Error`],
//! [`Event`], [`Timestamp`], [`LogEntry`], the [`Millivolts`]/[`Percent`]
//! units, and the [`RenderPolicy`]. Depends on nothing.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod datetime;
mod error;
mod event;
mod log_entry;
mod mode;
mod render_policy;
mod time;
mod units;

pub use datetime::{is_supported_year, DateTime};
pub use error::Error;
pub use event::{Event, WakeReason};
pub use log_entry::{LogEntry, SCHEMA_VERSION};
pub use mode::Mode;
pub use render_policy::{ColorMode, RenderPolicy};
pub use time::Timestamp;
pub use units::{Millivolts, Percent};
