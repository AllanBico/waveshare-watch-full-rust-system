//! VPS sync protocol.
//!
//! Checkpoint tracking and conflict resolution over records read from
//! `storage`. Active in desk mode only.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
