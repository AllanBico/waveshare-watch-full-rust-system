//! Runtime logging.
//!
//! A retained RAM ring buffer for wake reasons, battery samples, mode
//! switches, and errors. The buffer lives in RTC slow memory in firmware and
//! survives deep sleep. Flushing to the time-series layer is the caller's job;
//! this crate only stores and returns records.
//!
//! The [`log!`] macro expands to nothing unless the caller enables its
//! `logging` feature.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod ring;
mod wake;

pub use ring::{Retained, Ring, CAPACITY, RETAINED_BYTES};
pub use wake::wake_entries;

/// Appends an entry only when the caller's `logging` feature is enabled.
///
/// Expands to nothing otherwise, so disabled builds do not evaluate the ring
/// or entry expressions.
#[macro_export]
macro_rules! log {
    ($ring:expr, $entry:expr) => {{
        #[cfg(feature = "logging")]
        {
            let _ = $ring.push(&$entry);
        }
    }};
}
