//! Persistent one-shot watch alarms.
//!
//! Alarms are device state, not settings. The set is fixed-capacity and sorted
//! so the next wake and any missed alarm are cheap to find without allocation.
//! Persistence sits over [`hal::Storage`]; firmware supplies a TicKV backend.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod set;
mod store;

pub use set::{AlarmSet, MAX_ALARMS};
pub use store::{AlarmStore, LoadReport, StoreError, ALARM_KEY};
