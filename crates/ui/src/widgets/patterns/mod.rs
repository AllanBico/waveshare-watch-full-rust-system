//! Reusable screen patterns.

use common::Region;

use crate::assets::IconId;
use crate::layout::{Size, UNBOUNDED};
use crate::painter::Painter;
use crate::text::{measure_text, TextCache};
use crate::theme::Theme;

use super::{Widget, WidgetKind};

mod feedback;
mod screen;

/// Props for a titled screen body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenScaffoldProps<'a> {
    /// Screen title.
    pub title: &'a str,
    /// Screen body.
    pub body: &'a str,
}

/// Props for a centered empty state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmptyStateProps<'a> {
    /// Leading icon.
    pub icon: IconId,
    /// State title.
    pub title: &'a str,
    /// Supporting text.
    pub body: &'a str,
}

/// Props for a loading state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LoadingStateProps<'a> {
    /// Loading label.
    pub label: &'a str,
}

/// Props for an error state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ErrorStateProps<'a> {
    /// Error title.
    pub title: &'a str,
    /// Error detail.
    pub body: &'a str,
    /// Retry action label.
    pub retry: &'a str,
}

/// Props for a confirmation prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConfirmProps<'a> {
    /// Prompt title.
    pub title: &'a str,
    /// Prompt body.
    pub body: &'a str,
    /// Confirm action label.
    pub confirm: &'a str,
    /// Cancel action label.
    pub cancel: &'a str,
}

/// Measures a pattern.
pub(crate) fn measure(
    kind: &WidgetKind<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    match kind {
        WidgetKind::ScreenScaffold(props) => measure_scaffold(props, theme, cache, available),
        WidgetKind::EmptyState(props) => measure_empty(props, theme, cache, available),
        WidgetKind::LoadingState(props) => measure_loading(props, theme, cache, available),
        WidgetKind::ErrorState(props) => measure_error(props, theme, cache, available),
        WidgetKind::Confirm(props) => measure_confirm(props, theme, cache, available),
        _ => Size::ZERO,
    }
}

fn measure_scaffold(
    props: &ScreenScaffoldProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let title = measure_text(props.title, &theme.type_scale.title, 0, cache);
    let body = measure_text(props.body, &theme.type_scale.body, 0, cache);
    let width = if available == UNBOUNDED {
        320
    } else {
        available
    };
    let height = title
        .height
        .saturating_add(theme.spacing.md)
        .saturating_add(body.height)
        .saturating_add(theme.spacing.lg.saturating_mul(2));
    Size::new(width, height)
}

fn measure_empty(
    props: &EmptyStateProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let title = measure_text(props.title, &theme.type_scale.title, 0, cache);
    let body = measure_text(props.body, &theme.type_scale.body, 0, cache);
    let width = if available == UNBOUNDED {
        300
    } else {
        available
    };
    let height = theme
        .icons
        .lg
        .saturating_add(theme.spacing.md)
        .saturating_add(title.height)
        .saturating_add(theme.spacing.sm)
        .saturating_add(body.height)
        .saturating_add(theme.spacing.lg.saturating_mul(2));
    Size::new(width, height)
}

fn measure_loading(
    _props: &LoadingStateProps<'_>,
    theme: &Theme,
    _cache: &mut TextCache,
    available: u16,
) -> Size {
    let width = if available == UNBOUNDED {
        200
    } else {
        available
    };
    Size::new(
        width,
        theme
            .icons
            .lg
            .saturating_add(theme.spacing.md)
            .saturating_add(theme.type_scale.caption.line_height_px)
            .saturating_add(theme.spacing.lg.saturating_mul(2)),
    )
}

fn measure_error(
    props: &ErrorStateProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let title = measure_text(props.title, &theme.type_scale.title, 0, cache);
    let body = measure_text(props.body, &theme.type_scale.body, 0, cache);
    let width = if available == UNBOUNDED {
        320
    } else {
        available
    };
    let height = theme
        .icons
        .lg
        .saturating_add(theme.spacing.md)
        .saturating_add(title.height)
        .saturating_add(theme.spacing.sm)
        .saturating_add(body.height)
        .saturating_add(theme.min_touch_px)
        .saturating_add(theme.spacing.lg.saturating_mul(2));
    Size::new(width, height)
}

fn measure_confirm(
    props: &ConfirmProps<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Size {
    let title = measure_text(props.title, &theme.type_scale.title, 0, cache);
    let body = measure_text(props.body, &theme.type_scale.body, 0, cache);
    let width = if available == UNBOUNDED {
        360
    } else {
        available
    };
    let height = title
        .height
        .saturating_add(theme.spacing.sm)
        .saturating_add(body.height)
        .saturating_add(theme.min_touch_px)
        .saturating_add(theme.spacing.lg.saturating_mul(2));
    Size::new(width, height)
}

/// Paints a pattern.
pub(crate) fn paint(painter: &mut Painter<'_>, widget: &Widget<'_>, rect: Region, theme: &Theme) {
    match &widget.kind {
        WidgetKind::ScreenScaffold(_) | WidgetKind::EmptyState(_) | WidgetKind::LoadingState(_) => {
            screen::paint(painter, widget, rect, theme)
        }
        WidgetKind::ErrorState(_) | WidgetKind::Confirm(_) => {
            feedback::paint(painter, widget, rect, theme)
        }
        _ => {}
    }
}
