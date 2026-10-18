//! Buttons and icon buttons.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::Size;
use crate::painter::Painter;
use crate::text::{draw_text, measure_text, measure_uncached, TextAlign, TextCache};
use crate::theme::Theme;

use super::paint::{
    centered_offset, colors, content, draw_centered, draw_icon_centered, frame, Colors,
};
use super::{Widget, WidgetKind, WidgetState};

/// Props for a labelled action button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ButtonProps<'a> {
    /// Button label.
    pub label: &'a str,
    /// Optional leading icon.
    pub icon: Option<IconId>,
}

/// Props for a square icon-only button.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IconButtonProps<'a> {
    /// Icon to draw.
    pub icon: IconId,
    /// Accessibility label; not drawn.
    pub label: &'a str,
}

/// Measures a button or icon button.
pub(crate) fn measure(
    kind: &WidgetKind<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    _available: u16,
) -> Size {
    match kind {
        WidgetKind::Button(props) => {
            let metrics = measure_text(props.label, &theme.type_scale.label, 0, cache);
            let icon = if props.icon.is_some() {
                theme.icons.sm + theme.spacing.sm
            } else {
                0
            };
            let inset = inset(theme);
            Size::new(
                metrics.width.saturating_add(icon).saturating_add(inset),
                theme.min_touch_px.max(metrics.height.saturating_add(inset)),
            )
        }
        WidgetKind::IconButton(_) => Size::new(theme.min_touch_px, theme.min_touch_px),
        _ => Size::ZERO,
    }
}

fn inset(theme: &Theme) -> u16 {
    let edge = theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(theme.spacing.sm);
    edge.saturating_mul(2)
}

/// Paints a button or icon button.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    match &widget.kind {
        WidgetKind::Button(props) => paint_button(painter, props, rect, theme, widget.state),
        WidgetKind::IconButton(props) => paint_icon_button(painter, props, rect, theme, widget),
        _ => {}
    }
}

fn paint_button(
    painter: &mut Painter<'_>,
    props: &ButtonProps<'_>,
    rect: Region,
    theme: &Theme,
    state: WidgetState,
) {
    frame(painter, rect, theme, state, theme.radii.md);
    let colors = colors(theme, state);
    let region = content(rect, theme, theme.spacing.sm);
    if let Some(icon) = props.icon {
        paint_icon_label(painter, props.label, icon, region, theme, colors);
    } else {
        draw_centered(
            painter,
            props.label,
            &theme.type_scale.label,
            region,
            colors.content,
        );
    }
}

fn paint_icon_label(
    painter: &mut Painter<'_>,
    label: &str,
    icon: IconId,
    region: Region,
    theme: &Theme,
    colors: Colors,
) {
    let style = theme.type_scale.label;
    let text = measure_uncached(label, &style, 0);
    let icon_size = theme.icons.sm;
    let gap = theme.spacing.sm;
    let total = icon_size.saturating_add(gap).saturating_add(text.width);
    let start = region
        .x
        .saturating_add(centered_offset(region.width, total));
    let icon_y = region
        .y
        .saturating_add(centered_offset(region.height, icon_size));
    draw_icon(painter, icon, icon_size, (start, icon_y), colors.content);
    let text_x = start.saturating_add(icon_size).saturating_add(gap);
    let text_y = region
        .y
        .saturating_add(centered_offset(region.height, text.height));
    draw_text(
        painter,
        label,
        &style,
        (text_x, text_y),
        0,
        TextAlign::Start,
        colors.content,
    );
}

fn paint_icon_button(
    painter: &mut Painter<'_>,
    props: &IconButtonProps<'_>,
    rect: Region,
    theme: &Theme,
    widget: &Widget<'_>,
) {
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.sm);
    let size = theme.icons.lg.min(region.width).min(region.height);
    draw_icon_centered(painter, props.icon, size, region, colors.content);
    let _ = props.label;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::MonoSurface;
    use crate::theme::Theme;

    #[test]
    fn button_measure_clears_the_touch_target() {
        let mut cache = TextCache::new();
        let kind = WidgetKind::Button(ButtonProps {
            label: "Save",
            icon: Some(IconId::Check),
        });
        let size = measure(&kind, &Theme::WATCH, &mut cache, 0);
        assert!(size.width > 0);
        assert!(size.height >= Theme::WATCH.min_touch_px);
    }

    #[test]
    fn pressed_and_normal_buttons_differ() {
        let mut data = vec![0u8; MonoSurface::buffer_len(120, 88)];
        let mut surface = MonoSurface::new(120, 88, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            let normal = Widget::new(
                1,
                WidgetKind::Button(ButtonProps {
                    label: "Go",
                    icon: None,
                }),
            );
            paint(&mut painter, &normal, Region::full(120, 88), &Theme::WATCH);
        }
        let normal_lit = surface.lit_pixels();
        let mut data = vec![0u8; MonoSurface::buffer_len(120, 88)];
        let mut surface = MonoSurface::new(120, 88, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            let pressed = Widget::new(
                1,
                WidgetKind::Button(ButtonProps {
                    label: "Go",
                    icon: None,
                }),
            )
            .with_state(WidgetState::PRESSED);
            paint(&mut painter, &pressed, Region::full(120, 88), &Theme::WATCH);
        }
        assert_ne!(surface.lit_pixels(), normal_lit);
    }

    #[test]
    fn shrink_never_underflows() {
        assert!(super::super::paint::shrink(Region::new(0, 0, 4, 4), 9).is_empty());
    }
}
