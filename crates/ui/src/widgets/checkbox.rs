//! Checkboxes.

use common::Region;

use crate::assets::IconId;
use crate::icon::draw_icon;
use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{measure_text, TextCache};
use crate::theme::Theme;

use super::paint::{centered_offset, colors, content, draw_start, frame, on, shrink};
use super::{Widget, WidgetKind};

/// Checkbox side length in pixels.
pub(crate) const BOX_SIZE: u16 = 28;

/// Props for an independent checkbox.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CheckboxProps<'a> {
    /// Whether the box is checked.
    pub checked: bool,
    /// Row label.
    pub label: &'a str,
}

/// Measures a checkbox row.
pub(crate) fn measure(
    props: &CheckboxProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let label = measure_text(props.label, &theme.type_scale.body, 0, cache);
    let inset = inset(theme, theme.spacing.md);
    let width = if available == UNBOUNDED {
        240
    } else {
        available
    };
    Size::new(
        width,
        theme.min_touch_px.max(label.height.saturating_add(inset)),
    )
}

pub(crate) fn inset(theme: &Theme, padding: u16) -> u16 {
    theme
        .borders
        .thick
        .saturating_add(1)
        .saturating_add(padding)
        .saturating_mul(2)
}

/// Paints a checkbox row.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    let WidgetKind::Checkbox(props) = &widget.kind else {
        return;
    };
    frame(painter, rect, theme, widget.state, theme.radii.md);
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.md);
    let size = BOX_SIZE;
    let box_region = Region::new(
        region.x,
        region.y + centered_offset(region.height, size),
        size.min(region.width),
        size.min(region.height),
    );
    paint_box(painter, box_region, props.checked, theme, colors.content);
    let label_region = Region::new(
        box_region.right().saturating_add(theme.spacing.md),
        region.y,
        region
            .right()
            .saturating_sub(box_region.right().saturating_add(theme.spacing.md)),
        region.height,
    );
    draw_start(
        painter,
        props.label,
        &theme.type_scale.body,
        label_region,
        colors.content,
    );
}

pub(crate) fn paint_box(
    painter: &mut Painter<'_>,
    rect: Region,
    checked: bool,
    theme: &Theme,
    active: crate::luma::Luma,
) {
    let radius = theme.radii.sm.min(rect.height / 2);
    let border = theme.borders.thick.max(1);
    if checked {
        painter.fill_rounded_rect(rect, radius, active);
        let icon = theme.icons.sm.min(rect.width).min(rect.height);
        let inner = shrink(rect, theme.spacing.xxs);
        let x = inner.x + centered_offset(inner.width, icon);
        let y = inner.y + centered_offset(inner.height, icon);
        draw_icon(painter, IconId::Check, icon, (x, y), on(theme, active));
    } else {
        painter.fill_rounded_rect(rect, radius, active);
        painter.fill_rounded_rect(
            shrink(rect, border),
            radius.saturating_sub(border),
            theme.tones.background,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::MonoSurface;

    #[test]
    fn a_checked_box_lights_the_check_mark() {
        let mut data = vec![0u8; MonoSurface::buffer_len(200, 88)];
        let mut surface = MonoSurface::new(200, 88, &mut data).unwrap();
        {
            let mut painter = Painter::new(&mut surface);
            let widget = Widget::new(
                1,
                WidgetKind::Checkbox(CheckboxProps {
                    checked: true,
                    label: "Sync",
                }),
            );
            paint(&mut painter, &widget, Region::full(200, 88), &Theme::WATCH);
        }
        assert!(surface.lit_pixels() > 0);
    }

    #[test]
    fn label_metrics_are_used() {
        let mut cache = TextCache::new();
        let size = measure(
            &CheckboxProps {
                checked: false,
                label: "A longer label",
            },
            &Theme::WATCH,
            &mut cache,
            240,
        );
        assert_eq!(size.width, 240);
    }
}
