//! Screen patterns.

use common::Region;

use crate::assets::IconId;
use crate::painter::Painter;
use crate::text::{draw_text, measure_uncached, TextAlign};
use crate::theme::Theme;

use super::super::paint::{centered_offset, colors, content, draw_icon_centered, focus_ring};
use super::super::{Widget, WidgetKind};
use super::{EmptyStateProps, LoadingStateProps, ScreenScaffoldProps};

pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    match &widget.kind {
        WidgetKind::ScreenScaffold(props) => paint_scaffold(painter, props, rect, theme),
        WidgetKind::EmptyState(props) => paint_empty(painter, widget, props, rect, theme),
        WidgetKind::LoadingState(props) => paint_loading(painter, widget, props, rect, theme),
        _ => {}
    }
}

fn paint_scaffold(
    painter: &mut Painter<'_>,
    props: &ScreenScaffoldProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    painter.fill_rect(rect, theme.tones.background);
    let region = content(rect, theme, theme.spacing.lg);
    let title = measure_uncached(props.title, &theme.type_scale.title, region.width);
    draw_text(
        painter,
        props.title,
        &theme.type_scale.title,
        (region.x, region.y),
        region.width,
        TextAlign::Start,
        theme.tones.text,
    );
    if props.body.is_empty() {
        return;
    }
    let divider = region
        .y
        .saturating_add(title.height)
        .saturating_add(theme.spacing.md);
    painter.fill_rect(
        Region::new(region.x, divider, region.width, theme.borders.thin.max(1)),
        theme.tones.border,
    );
    draw_text(
        painter,
        props.body,
        &theme.type_scale.body,
        (region.x, divider.saturating_add(theme.spacing.md)),
        region.width,
        TextAlign::Start,
        theme.tones.text,
    );
}

fn paint_empty(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &EmptyStateProps<'_>,
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
    let top = region.y + centered_offset(region.height, total);
    draw_icon_centered(
        painter,
        props.icon,
        icon,
        Region::new(region.x, top, region.width, icon),
        colors.content,
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
    focus_ring(painter, rect, theme, widget.state);
}

fn paint_loading(
    painter: &mut Painter<'_>,
    widget: &Widget<'_>,
    props: &LoadingStateProps<'_>,
    rect: Region,
    theme: &Theme,
) {
    let colors = colors(theme, widget.state);
    let region = content(rect, theme, theme.spacing.lg);
    let icon = theme.icons.lg;
    let label = measure_uncached(props.label, &theme.type_scale.caption, region.width);
    let total = icon
        .saturating_add(theme.spacing.md)
        .saturating_add(label.height);
    let top = region.y + centered_offset(region.height, total);
    draw_icon_centered(
        painter,
        IconId::ArrowClockwise,
        icon,
        Region::new(region.x, top, region.width, icon),
        colors.content,
    );
    draw_text(
        painter,
        props.label,
        &theme.type_scale.caption,
        (region.x, top + icon + theme.spacing.md),
        region.width,
        TextAlign::Center,
        colors.content,
    );
    focus_ring(painter, rect, theme, widget.state);
}
