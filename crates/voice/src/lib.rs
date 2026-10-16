//! Voice pipeline: wake word, audio capture, and the VPS client.
//!
//! Desk mode only. The wake word is a hand-rolled `no_std` pipeline gated by
//! VAD. Opus and endpointing live on the VPS.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
