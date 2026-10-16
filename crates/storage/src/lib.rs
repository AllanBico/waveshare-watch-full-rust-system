//! Persistent storage.
//!
//! Wraps TicKV with a raw KV API and a time-series API on top. Owns the flash
//! partition and the retention policy.

#![no_std]
