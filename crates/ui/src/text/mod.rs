//! Compiled-in text measurement and rendering.

mod cache;
mod draw;
mod font;
mod layout;

pub use cache::{TextCache, TEXT_CACHE_CAPACITY};
pub use draw::{draw_text, measure_text, measure_uncached, TextAlign};
pub use font::{font_asset, FontAsset, Glyph};
pub use layout::{
    ellipsis_prefix, layout_text, measure_layout, text_width, Line, LineLayout, TextMetrics,
    MAX_LINES,
};
