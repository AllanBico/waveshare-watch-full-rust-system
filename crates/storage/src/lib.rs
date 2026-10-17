//! Persistent key-value storage.
//!
//! Wraps TicKV over a `NorFlash` partition. The flash device and partition
//! table arrive from `drivers`; this crate owns hashing, provisioning policy,
//! TicKV error mapping, and the time-series layer added in Phase 2.2.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod controller;
mod error;
mod hash;
mod kv;
mod time_series;

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod time_series_tests;

pub use controller::REGION_SIZE;
pub use error::Error;
pub use hash::{hash_key, MAIN_KEY_HASH};
pub use kv::{OpenOutcome, Provisioning, TicKvStorage, MAX_KEY_LEN, MAX_VALUE_LEN, SCHEMA_VERSION};
pub use time_series::TimeSeries;
