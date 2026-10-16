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
mod font;
mod mono;

pub use clock::{clock_region, clock_size, draw_time};
pub use mono::MonoBuffer;
