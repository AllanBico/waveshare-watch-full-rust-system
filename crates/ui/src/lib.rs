//! The custom UI engine.
//!
//! Theme tokens, widget tree, layout, renderer, dirty-rect diffing, and
//! animation. Renders into a `Surface` under a runtime `RenderPolicy`.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
