//! Text measurement and wrapping.

use super::font::FontAsset;

/// Maximum lines retained by one layout.
pub const MAX_LINES: usize = 16;

/// Measured text size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextMetrics {
    /// Widest line in pixels.
    pub width: u16,
    /// Total height in pixels.
    pub height: u16,
    /// Number of lines.
    pub line_count: u16,
}

impl TextMetrics {
    /// An empty measurement.
    pub const ZERO: Self = Self {
        width: 0,
        height: 0,
        line_count: 0,
    };
}

/// One laid-out line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line<'a> {
    /// Line text without surrounding whitespace.
    pub text: &'a str,
    /// Line width in pixels.
    pub width: u16,
}

/// A fixed-capacity wrapped layout.
#[derive(Debug, Clone, Copy)]
pub struct LineLayout<'a> {
    lines: [Line<'a>; MAX_LINES],
    count: usize,
    truncated: bool,
}

impl<'a> LineLayout<'a> {
    /// The laid-out lines.
    pub fn lines(&self) -> &[Line<'a>] {
        &self.lines[..self.count]
    }

    /// Whether text remained after the line cap.
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }
}

/// Lays out `text` into lines no wider than `max_width`.
///
/// `max_width == 0` means no wrapping. Words longer than the line are clipped
/// by the painter; use [`ellipsis_prefix`] for single-line truncation.
pub fn layout_text<'a>(text: &'a str, asset: &FontAsset, max_width: u16) -> LineLayout<'a> {
    let mut result = LineLayout {
        lines: [Line { text: "", width: 0 }; MAX_LINES],
        count: 0,
        truncated: false,
    };
    if text.is_empty() {
        return result;
    }
    for paragraph in text.split('\n') {
        if !wrap_paragraph(&mut result, paragraph, asset, max_width) {
            break;
        }
    }
    result
}

/// Measures a layout at a given line height.
pub fn measure_layout(layout: &LineLayout<'_>, line_height_px: u16) -> TextMetrics {
    let mut width = 0;
    for line in layout.lines() {
        width = width.max(line.width);
    }
    TextMetrics {
        width,
        height: line_height_px.saturating_mul(layout.count as u16),
        line_count: layout.count as u16,
    }
}

/// Returns the longest prefix of `text` that fits with `...` appended.
pub fn ellipsis_prefix<'a>(text: &'a str, asset: &FontAsset, max_width: u16) -> &'a str {
    if text_width(text, asset) <= u32::from(max_width) {
        return text;
    }
    let dots = text_width("...", asset);
    if dots > u32::from(max_width) {
        return "";
    }
    let mut width = 0u32;
    let mut end = 0usize;
    for (offset, character) in text.char_indices() {
        if character == '\n' {
            end = offset;
            break;
        }
        let advance = u32::from(asset.advance(character));
        if width + advance + dots > u32::from(max_width) {
            end = offset;
            break;
        }
        width += advance;
        end = offset + character.len_utf8();
    }
    &text[..end]
}

/// The width of one run of text, ignoring newlines.
pub fn text_width(text: &str, asset: &FontAsset) -> u32 {
    text.chars()
        .filter(|character| *character != '\n')
        .map(|character| u32::from(asset.advance(character)))
        .sum()
}

fn wrap_paragraph<'a>(
    result: &mut LineLayout<'a>,
    paragraph: &'a str,
    asset: &FontAsset,
    max_width: u16,
) -> bool {
    if paragraph.is_empty() {
        return push_line(result, "", 0);
    }
    if max_width == 0 {
        let width = text_width(paragraph, asset);
        return push_line(result, paragraph, width);
    }
    let mut line_start = None;
    let mut line_end = 0usize;
    let mut line_width = 0u32;
    for (start, end) in words(paragraph) {
        let word_width = text_width(&paragraph[start..end], asset);
        let space = if line_start.is_some() {
            u32::from(asset.advance(' '))
        } else {
            0
        };
        let candidate = line_width + space + word_width;
        if line_start.is_some() && candidate > u32::from(max_width) {
            let Some(first) = line_start.replace(start) else {
                return false;
            };
            if !push_line(result, &paragraph[first..line_end], line_width) {
                return false;
            }
            line_width = word_width;
            line_end = end;
        } else {
            if line_start.is_none() {
                line_start = Some(start);
            }
            line_width = candidate;
            line_end = end;
        }
    }
    match line_start {
        Some(start) => push_line(result, &paragraph[start..line_end], line_width),
        None => push_line(result, "", 0),
    }
}

fn push_line<'a>(result: &mut LineLayout<'a>, text: &'a str, width: u32) -> bool {
    if result.count == MAX_LINES {
        result.truncated = true;
        return false;
    }
    result.lines[result.count] = Line {
        text,
        width: width.min(u32::from(u16::MAX)) as u16,
    };
    result.count += 1;
    true
}

struct Words<'a> {
    text: &'a str,
    position: usize,
}

fn words(text: &str) -> Words<'_> {
    Words { text, position: 0 }
}

impl<'a> Iterator for Words<'a> {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let bytes = self.text.as_bytes();
        while self.position < bytes.len() && bytes[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
        if self.position >= bytes.len() {
            return None;
        }
        let start = self.position;
        while self.position < bytes.len() && !bytes[self.position].is_ascii_whitespace() {
            self.position += 1;
        }
        Some((start, self.position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::font_asset;
    use crate::theme::{FontId, Weight};

    fn asset() -> &'static FontAsset {
        font_asset(FontId::Ui, Weight::Regular, 22).unwrap()
    }

    #[test]
    fn empty_text_has_no_lines() {
        assert_eq!(layout_text("", asset(), 100).lines().len(), 0);
    }

    #[test]
    fn short_text_is_one_line() {
        let lines = layout_text("Hello", asset(), 0);
        assert_eq!(lines.lines().len(), 1);
        assert_eq!(lines.lines()[0].text, "Hello");
        assert!(lines.lines()[0].width > 0);
    }

    #[test]
    fn wraps_at_word_boundaries() {
        let lines = layout_text("hello world again", asset(), 70);
        assert!(lines.lines().len() > 1);
        for line in lines.lines() {
            assert!(line.width <= 70 || !line.text.contains(' '));
        }
    }

    #[test]
    fn explicit_newlines_are_preserved() {
        let lines = layout_text("one\ntwo", asset(), 0);
        assert_eq!(lines.lines().len(), 2);
        assert_eq!(lines.lines()[0].text, "one");
        assert_eq!(lines.lines()[1].text, "two");
    }

    #[test]
    fn ellipsis_prefix_fits_with_dots() {
        let prefix = ellipsis_prefix("a very long label", asset(), 40);
        let width = text_width(prefix, asset()) + text_width("...", asset());
        assert!(width <= 40);
        assert!(prefix.len() < "a very long label".len());
    }
}
