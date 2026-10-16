//! Shared vocabulary for the whole system.
//!
//! Owns the types every other crate depends on: [`Mode`], [`Error`],
//! [`Event`], [`Timestamp`], and [`LogEntry`]. Depends on nothing.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod error;
mod event;
mod log_entry;
mod mode;
mod time;

pub use error::Error;
pub use event::{Event, WakeReason};
pub use log_entry::{LogEntry, SCHEMA_VERSION};
pub use mode::Mode;
pub use time::Timestamp;
