//! Shared vocabulary for the whole system.
//!
//! Owns the types every other crate depends on: [`Mode`], [`Error`],
//! [`Event`], [`Timestamp`], [`LogEntry`], the [`Millivolts`]/[`Percent`]
//! units, the [`RenderPolicy`], and the [`Region`]/[`Rgb565`] pixel types.
//! Depends on nothing.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod color;
mod datetime;
mod error;
mod event;
mod geometry;
mod log_codec;
mod log_entry;
mod mode;
mod render_policy;
mod time;
mod touch;
mod units;

pub use color::Rgb565;
pub use datetime::{is_supported_year, DateTime};
pub use error::Error;
pub use event::{Event, WakeReason};
pub use geometry::Region;
pub use log_codec::{decode_log, encode_log, LogError, MAX_ENCODED_LOG_LEN};
pub use log_entry::{LogEntry, SCHEMA_VERSION};
pub use mode::Mode;
pub use render_policy::{ColorMode, RenderPolicy};
pub use time::Timestamp;
pub use touch::{TouchPoint, TouchReport, MAX_TOUCH_POINTS};
pub use units::{Millivolts, Percent};
