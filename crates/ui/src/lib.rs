//! The custom UI engine.
//!
//! Theme tokens, widget tree, layout, renderer, dirty-rect diffing, and
//! animation. Renders into a `Surface` under a runtime `RenderPolicy`.
//!
//! Phase 3.3 ships only the monochrome render target: the [`MonoBuffer`], a
//! temporary clock renderer, and the lit-pixel accounting the render policy
//! budgets against.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod clock;
#[cfg(feature = "debug-text")]
mod debug;
mod font;
mod mono;

pub use clock::{clock_region, clock_size, digits_size, draw_digits, draw_time};
#[cfg(feature = "debug-text")]
pub use debug::{draw_text, text_size};
pub use mono::MonoBuffer;
