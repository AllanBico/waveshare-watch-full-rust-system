//! Damage repair and display flush.

mod flush;
mod paint;
#[cfg(test)]
mod tests;

pub use flush::{flush, FlushError, Rgb565Source};
pub use paint::{paint_damage, paint_damage_shaped};
