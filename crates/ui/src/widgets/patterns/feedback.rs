//! Feedback patterns.

use common::Region;

use crate::assets::IconId;
use crate::painter::Painter;
use crate::text::{draw_text, measure_uncached, TextAlign};
use crate::theme::Theme;

use super::super::paint::{
    centered_offset, colors, content, draw_centered, draw_icon_centered, focus_ring, frame, on,
    shrink,
};
use super::super::{Widget, WidgetKind, WidgetState};
use super::{ConfirmProps, ErrorStateProps};

pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    match &widget.kind {
        WidgetKind::ErrorState(props) => paint_error(painter, widget, props, rect, theme),
        WidgetKind::Confirm(props) => paint_confirm(painter, widget, props, rect, theme),
        _ => {}
    }
}

fn paint_error(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &ErrorStateProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.lg);
    let icon = theme.icons.lg;
    let title = measure_uncached(props.title, &theme.type_scale.title, region.width);
    let body = measure_uncached(props.body, &theme.type_scale.body, region.width);
    let total = icon
        .saturating_add(theme.spacing.md)
        .saturating_add(title.height)
        .saturating_add(theme.spacing.sm)
        .saturating_add(body.height);
    let top = region.y + centered_offset(region.height.saturating_sub(theme.min_touch_px), total);
    draw_icon_centered(
        painter,
        IconId::Warning,
        icon,
        Region::new(region.x, top, region.width, icon),
        theme.tones.error,
    );
    let title_top = top + icon + theme.spacing.md;
    draw_text(
        painter,
        props.title,
        &theme.type_scale.title,
        (region.x, title_top),
        region.width,
        TextAlign::Center,
        colors.content,
    );
    draw_text(
        painter,
        props.body,
        &theme.type_scale.body,
        (region.x, title_top + title.height + theme.spacing.sm),
        region.width,
        TextAlign::Center,
        colors.content,
    );
    let action = Region::new(
        region.x,
        region.bottom().saturating_sub(theme.min_touch_px),
        region.width,
        theme.min_touch_px,
    );
    paint_action(painter, action, props.retry, theme, false);
    focus_ring(painter, rect, theme, widget.state);
}

fn paint_confirm(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &ConfirmProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.lg);
    let title = measure_uncached(props.title, &theme.type_scale.title, region.width);
    draw_text(
        painter,
        props.title,
        &theme.type_scale.title,
        (region.x, region.y),
        region.width,
        TextAlign::Start,
        colors.content,
    );
    draw_text(
        painter,
        props.body,
        &theme.type_scale.body,
        (region.x, region.y + title.height + theme.spacing.sm),
        region.width,
        TextAlign::Start,
        colors.content,
    );
    let gap = theme.spacing.md;
    let width = region.width.saturating_sub(gap) / 2;
    let top = region.bottom().saturating_sub(theme.min_touch_px);
    let cancel = Region::new(region.x, top, width, theme.min_touch_px);
    let confirm = Region::new(region.x + width + gap, top, width, theme.min_touch_px);
    paint_action(painter, cancel, props.cancel, theme, false);
    paint_action(painter, confirm, props.confirm, theme, true);
    focus_ring(painter, rect, theme, widget.state);
}

fn paint_action(
    painter: &mut Painter<'_>,
    region: Region,
    label: &str,
    theme: &Theme,
    filled: bool,
) {
    if filled {
        let radius = theme.radii.md;
        painter.fill_rounded_rect(region, radius, theme.tones.selected);
        let color = on(theme, theme.tones.selected);
        draw_centered(
            painter,
            label,
            &theme.type_scale.label,
            shrink(region, theme.spacing.md),
            color,
        );
    } else {
        frame(painter, region, theme, WidgetState::NORMAL, theme.radii.md);
        let color = theme.tones.text;
        draw_centered(
            painter,
            label,
            &theme.type_scale.label,
            shrink(region, theme.spacing.md),
            color,
        );
    }
}
