//! Voice pipeline: wake word, audio capture, and the VPS client.
//!
//! Desk mode only. The wake word is a hand-rolled `no_std` pipeline gated by
//! VAD. Opus and endpointing live on the VPS.

#![no_std]
