//! Persistent storage.
//!
//! Wraps TicKV with a raw KV API and a time-series API on top. Owns the flash
//! partition and the retention policy.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
