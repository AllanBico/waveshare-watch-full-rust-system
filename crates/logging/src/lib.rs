//! Runtime logging.
//!
//! A RAM ring buffer for wake reasons, mode switches, and errors. Flushes to
//! `storage` only on significant events. Compiles to nothing when the
//! `logging` feature is off.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
