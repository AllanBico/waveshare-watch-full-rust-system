//! Settings rows with a trailing control or value.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, frame};
use super::{checkbox, toggle, Widget, WidgetKind};

const UNBOUNDED_WIDTH: u16 = 240;
const GAP: u16 = 8;

/// The trailing content of a settings row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsTrailing<'a> {
    /// Nothing after the text.
    None,
    /// A disclosure chevron.
    Chevron,
    /// A short read-only value.
    Value(&'a str),
    /// An on/off switch.
    Toggle(bool),
    /// An independent checkbox.
    Checkbox(bool),
}

/// Props for one settings row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SettingsRowProps<'a> {
    /// Primary label.
    pub title: &'a str,
    /// Optional secondary label.
    pub subtitle: &'a str,
    /// Optional leading icon.
    pub icon: Option<IconId>,
    /// Trailing control or value.
    pub trailing: SettingsTrailing<'a>,
    /// Uses the smaller label style for dense lists.
    pub compact: bool,
}

/// Measures a settings row.
pub(crate) fn measure(
    props: &SettingsRowProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let style = if props.compact {
        &theme.type_scale.label
    } else {
        &theme.type_scale.body
    };
    let title = measure_text(props.title, style, 0, cache);
    let subtitle = measure_text(props.subtitle, &theme.type_scale.caption, 0, cache);
    let text_height = title.height.saturating_add(if props.subtitle.is_empty() {
        0
    } else {
        theme.spacing.xs.saturating_add(subtitle.height)
    });
    let inset = inset(theme, props);
    Size::new(
        if available == UNBOUNDED {
            UNBOUNDED_WIDTH
        } else {
            available
        },
        theme.min_touch_px.max(text_height.saturating_add(inset)),
    )
}

fn padding(props: &SettingsRowProps<'_>, theme: &Theme) -> u16 {
    if props.compact && !props.subtitle.is_empty() {
        theme.spacing.xxs
    } else {
        theme.spacing.md
    }
}

fn inset(theme: &Theme, props: &SettingsRowProps<'_>) -> u16 {
    theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(padding(props, theme))
        .saturating_mul(2)
}

/// Paints a settings row.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::SettingsRow(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let horizontal = theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.md);
    let vertical = theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(padding(props, theme));
    let region = Region::new(
        rect.x.saturating_add(horizontal),
        rect.y.saturating_add(vertical),
        rect.width.saturating_sub(horizontal.saturating_mul(2)),
        rect.height.saturating_sub(vertical.saturating_mul(2)),
    );
    let mut text_region = region;
    if let Some(icon) = props.icon {
        let size = theme.icons.md.min(region.height);
        let y = region.y + centered_offset(region.height, size);
        draw_icon(painter, icon, size, (region.x, y), colors.content);
        text_region.x = text_region.x.saturating_add(size + theme.spacing.md);
        text_region.width = text_region.width.saturating_sub(size + theme.spacing.md);
    }
    let trailing = trailing_width(props.trailing, theme);
    text_region.width = text_region.width.saturating_sub(trailing + GAP);
    paint_text(painter, props, text_region, theme, colors.content);
    paint_trailing(painter, props.trailing, region, theme, &colors);
}

fn paint_text(
    painter: &mut Painter<'_>,
    props: &SettingsRowProps<'_>,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let style = if props.compact {
        &theme.type_scale.label
    } else {
        &theme.type_scale.body
    };
    let title = measure_uncached(props.title, style, region.width);
    let subtitle = measure_uncached(props.subtitle, &theme.type_scale.caption, region.width);
    let gap = if props.subtitle.is_empty() {
        0
    } else {
        theme.spacing.xs
    };
    let total = title
        .height
        .saturating_add(gap)
        .saturating_add(subtitle.height);
    let top = region.y + centered_offset(region.height, total);
    draw_text(
        painter,
        props.title,
        style,
        (region.x, top),
        region.width,
        TextAlign::Start,
        color,
    );
    if !props.subtitle.is_empty() {
        draw_text(
            painter,
            props.subtitle,
            &theme.type_scale.caption,
            (region.x, top + title.height + gap),
            region.width,
            TextAlign::Start,
            color,
        );
    }
}

fn paint_trailing(
    painter: &mut Painter<'_>,
    trailing: SettingsTrailing<'_>,
    region: Region,
    theme: &Theme,
    colors: &super::paint::Colors,
) {
    match trailing {
        SettingsTrailing::None => {}
        SettingsTrailing::Chevron => paint_chevron(painter, region, theme, colors.content),
        SettingsTrailing::Value(value) => {
            paint_value(painter, value, region, theme, colors.content)
        }
        SettingsTrailing::Toggle(checked) => paint_toggle(painter, checked, region, theme, colors),
        SettingsTrailing::Checkbox(checked) => {
            paint_checkbox(painter, checked, region, theme, colors.content)
        }
    }
}

fn paint_chevron(
    painter: &mut Painter<'_>,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let size = theme.icons.sm.min(region.height);
    let x = region.right().saturating_sub(size);
    let y = region.y + centered_offset(region.height, size);
    draw_icon(painter, IconId::CaretRight, size, (x, y), color);
}

fn paint_value(
    painter: &mut Painter<'_>,
    value: &str,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let width = measure_uncached(value, &theme.type_scale.caption, region.width).width;
    let x = region.right().saturating_sub(width.min(region.width));
    let text = Region::new(x, region.y, width.min(region.width), region.height);
    let style = theme.type_scale.caption;
    let metrics = measure_uncached(value, &style, text.width);
    let y = text.y + centered_offset(text.height, metrics.height);
    draw_text(
        painter,
        value,
        &style,
        (text.x, y),
        text.width,
        TextAlign::End,
        color,
    );
}

fn paint_toggle(
    painter: &mut Painter<'_>,
    checked: bool,
    region: Region,
    theme: &Theme,
    colors: &super::paint::Colors,
) {
    let track = Region::new(
        region.right().saturating_sub(toggle::TRACK_WIDTH),
        region.y + centered_offset(region.height, toggle::TRACK_HEIGHT),
        toggle::TRACK_WIDTH.min(region.width),
        toggle::TRACK_HEIGHT.min(region.height),
    );
    toggle::paint_switch(painter, track, checked, theme, colors.content, colors.fill);
}

fn paint_checkbox(
    painter: &mut Painter<'_>,
    checked: bool,
    region: Region,
    theme: &Theme,
    color: crate::luma::Luma,
) {
    let size = checkbox::BOX_SIZE.min(region.width).min(region.height);
    let box_region = Region::new(
        region.right().saturating_sub(size),
        region.y + centered_offset(region.height, size),
        size,
        size,
    );
    checkbox::paint_box(painter, box_region, checked, theme, color);
}

fn trailing_width(trailing: SettingsTrailing<'_>, theme: &Theme) -> u16 {
    match trailing {
        SettingsTrailing::None => 0,
        SettingsTrailing::Chevron => theme.icons.sm,
        SettingsTrailing::Value(value) => {
            measure_uncached(value, &theme.type_scale.caption, 0).width
        }
        SettingsTrailing::Toggle(_) => toggle::TRACK_WIDTH,
        SettingsTrailing::Checkbox(_) => checkbox::BOX_SIZE,
    }
}
