//! The custom monochrome UI engine.
//!
//! Both runtime modes share one luminance painter. Watch renders into a 1-bit
//! surface, desk into an 8-bit grayscale surface. Themes, surfaces, drawing
//! primitives, compiled-in fonts, and compiled-in icons live here.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod assets;
mod bringup;
mod icon;
mod layout;
mod luma;
mod painter;
mod surface;
#[cfg(test)]
mod test_support;
mod text;
mod theme;

pub use assets::{IconId, IconMask, IconSet, ICON_COUNT, ICON_SETS};
pub use bringup::draw_bringup;
pub use icon::draw_icon;
pub use layout::{
    layout, Align, Axis, Constraints, Content, Insets, Kind, LayoutEntry, LayoutError, LayoutRect,
    Length, Node, NodeId, Size, MAX_DEPTH, UNBOUNDED,
};
pub use luma::Luma;
pub use painter::{Painter, MAX_CLIP_DEPTH};
pub use surface::{GraySurface, MonoSurface, Surface, SurfaceError};
pub use text::{
    draw_text, ellipsis_prefix, font_asset, layout_text, measure_layout, measure_text,
    measure_uncached, text_width, FontAsset, Glyph, Line, LineLayout, TextAlign, TextCache,
    TextMetrics, MAX_LINES, TEXT_CACHE_CAPACITY,
};
pub use theme::{
    Borders, FontId, IconSizes, Motion, Radii, Spacing, TextStyle, Theme, Tones, Typography, Weight,
};
