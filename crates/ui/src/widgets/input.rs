//! Text input fields.

use common::Region;

use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{font_asset, measure_text, text_width, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, draw_start, frame};
use super::{Widget, WidgetKind};

const MASK_LIMIT: usize = 64;

/// Props for a text display field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputProps<'a> {
    /// Current text.
    pub value: &'a str,
    /// Shown when the value is empty.
    pub placeholder: &'a str,
    /// Whether to render the value as asterisks.
    pub masked: bool,
    /// Caret position as a character index.
    pub caret: Option<u16>,
}

/// Measures an input field.
pub(crate) fn measure(
    props: &InputProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let text = if props.value.is_empty() {
        props.placeholder
    } else {
        props.value
    };
    let metrics = measure_text(text, &theme.type_scale.body, 0, cache);
    let inset = theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.md)
        .saturating_mul(2);
    let width = if available == UNBOUNDED {
        240
    } else {
        available
    };
    Size::new(
        width,
        theme.min_touch_px.max(metrics.height.saturating_add(inset)),
    )
}

/// Paints an input field.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Input(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.md);
    let style = theme.type_scale.body;
    let text_color = if props.value.is_empty() {
        if widget.state.is_selected() || widget.state.is_pressed() {
            colors.content
        } else {
            theme.tones.text_muted
        }
    } else {
        colors.content
    };
    if props.value.is_empty() {
        draw_start(painter, props.placeholder, &style, region, text_color);
        paint_caret(painter, props, region, theme, colors.content);
    } else if props.masked {
        let count = props.value.chars().count().min(MASK_LIMIT);
        let buffer = [b'*'; MASK_LIMIT];
        if let Ok(masked) = core::str::from_utf8(&buffer[..count]) {
            draw_start(painter, masked, &style, region, colors.content);
        }
        paint_caret(painter, props, region, theme, colors.content);
    } else {
        draw_start(painter, props.value, &style, region, colors.content);
        paint_caret(painter, props, region, theme, colors.content);
    }
}

fn paint_caret(
    painter: &mut Painter<'_>,
    props: &InputProps<'_>,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let Some(index) = props.caret else {
        return;
    };
    let style = theme.type_scale.body;
    let width = prefix_width(props, index, theme);
    let x = region
        .x
        .saturating_add(width)
        .min(region.right().saturating_sub(2));
    let height = style.line_height_px.min(region.height);
    let y = region.y + centered_offset(region.height, height);
    painter.fill_rect(Region::new(x, y, 2.min(region.width), height), color);
}

fn prefix_width(props: &InputProps<'_>, index: u16, theme: &Theme) -> u16 {
    let Some(asset) = font_asset(
        theme.type_scale.body.font,
        theme.type_scale.body.weight,
        theme.type_scale.body.size_px,
    ) else {
        return 0;
    };
    let text = if props.masked { "*" } else { props.value };
    let count = if props.masked {
        props.value.chars().count()
    } else {
        text.chars().count()
    };
    let index = usize::from(index).min(count);
    if props.masked {
        return text_width("*", asset).saturating_mul(index as u32) as u16;
    }
    let end = text
        .char_indices()
        .nth(index)
        .map(|(offset, _)| offset)
        .unwrap_or(text.len());
    text_width(&text[..end], asset) as u16
}
