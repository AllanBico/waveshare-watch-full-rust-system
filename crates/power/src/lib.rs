//! Power management.
//!
//! Owns all rails, sleep decisions, and the render policy. Single source of
//! truth for power state and mode transitions.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
