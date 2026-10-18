//! Fixed-capacity text measurement cache.

use super::layout::TextMetrics;
use crate::theme::TextStyle;

/// Cached measurements retained by one cache.
pub const TEXT_CACHE_CAPACITY: usize = 64;

#[derive(Debug, Clone, Copy)]
struct Entry {
    key: u64,
    metrics: TextMetrics,
}

/// A fixed-capacity, direct-mapped text measurement cache.
pub struct TextCache {
    entries: [Option<Entry>; TEXT_CACHE_CAPACITY],
}

impl TextCache {
    /// Creates an empty cache.
    pub const fn new() -> Self {
        Self {
            entries: [None; TEXT_CACHE_CAPACITY],
        }
    }

    /// Returns a cached measurement.
    pub fn get(&self, text: &str, style: &TextStyle, max_width: u16) -> Option<TextMetrics> {
        let key = key(text, style, max_width);
        let slot = (key as usize) % TEXT_CACHE_CAPACITY;
        self.entries[slot]
            .filter(|entry| entry.key == key)
            .map(|entry| entry.metrics)
    }

    /// Stores a measurement, replacing the slot's entry.
    pub fn insert(&mut self, text: &str, style: &TextStyle, max_width: u16, metrics: TextMetrics) {
        let key = key(text, style, max_width);
        let slot = (key as usize) % TEXT_CACHE_CAPACITY;
        self.entries[slot] = Some(Entry { key, metrics });
    }

    /// The number of entries currently in use.
    pub fn len(&self) -> usize {
        self.entries.iter().filter(|entry| entry.is_some()).count()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for TextCache {
    fn default() -> Self {
        Self::new()
    }
}

fn key(text: &str, style: &TextStyle, max_width: u16) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for &byte in text.as_bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    for value in [
        style.font as u8,
        style.weight as u8,
        (style.size_px & 0xFF) as u8,
        (style.size_px >> 8) as u8,
        (style.line_height_px & 0xFF) as u8,
        (style.line_height_px >> 8) as u8,
        (max_width & 0xFF) as u8,
        (max_width >> 8) as u8,
    ] {
        hash ^= u64::from(value);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn cache_round_trips_a_measurement() {
        let style = Theme::WATCH.type_scale.body;
        let metrics = TextMetrics {
            width: 12,
            height: 34,
            line_count: 1,
        };
        let mut cache = TextCache::new();
        assert!(cache.is_empty());
        assert_eq!(cache.get("hi", &style, 100), None);
        cache.insert("hi", &style, 100, metrics);
        assert_eq!(cache.get("hi", &style, 100), Some(metrics));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn different_keys_do_not_collide_in_the_same_slot() {
        let style = Theme::WATCH.type_scale.body;
        let metrics = TextMetrics {
            width: 1,
            height: 2,
            line_count: 1,
        };
        let mut cache = TextCache::new();
        cache.insert("a", &style, 10, metrics);
        assert_eq!(cache.get("b", &style, 10), None);
    }
}
