//! The custom monochrome UI engine.
//!
//! Both runtime modes share one luminance painter. Watch renders into a 1-bit
//! surface, desk into an 8-bit grayscale surface. Themes, surfaces, and drawing
//! primitives live here; text, layout, components, and the shell arrive in
//! later Phase 5 steps.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod bringup;
mod luma;
mod painter;
mod surface;
#[cfg(test)]
mod test_support;
mod theme;

pub use bringup::draw_bringup;
pub use luma::Luma;
pub use painter::{Painter, MAX_CLIP_DEPTH};
pub use surface::{GraySurface, MonoSurface, Surface, SurfaceError};
pub use theme::{
    Borders, FontId, Motion, Radii, Spacing, TextStyle, Theme, Tones, Typography, Weight,
};
