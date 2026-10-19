//! Frame building for the category list and paged log list.

use common::{LogEntry, Region};
use ui::{
    Align, IconId, LabelProps, Length, PageIndicatorProps, SettingsRowProps, SettingsTrailing,
    TextAlign, TextCache, TextRole, Theme, Widget, WidgetArena, WidgetKind,
};

use super::model::{LogCategory, LOGS_PER_PAGE};
use crate::view::{
    background, column, content_padding, crown, fill_node, grid, node, spacer, BuildError, Builder,
};

const BG_ID: u16 = 1_000;
const TITLE_ID: u16 = 1_002;
const CROWN_ID: u16 = 1_006;
const CONTENT_ID: u16 = 1_007;
const GRID_ID: u16 = 1_008;
const PAGE_ID: u16 = 1_009;
const BODY_ID: u16 = 1_010;
const EMPTY_ID: u16 = 1_030;
pub(super) const CATEGORY_BASE: u16 = 1_020;
const ROW_BASE: u16 = 1_040;

/// The category id for an index.
pub(super) const fn category_id(index: usize) -> u16 {
    CATEGORY_BASE + index as u16
}

/// The category index for a hit id.
pub(super) fn category_index(id: u16) -> Option<usize> {
    let index = usize::from(id.checked_sub(CATEGORY_BASE)?);
    (index < LogCategory::ALL.len()).then_some(index)
}

/// Builds the category selection screen.
pub(super) fn build_categories(
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'_>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let title = title("Logs");
    let rows = categories();
    let title_node = fill_node(&title, theme, cache, viewport.width);
    let row_nodes = rows.map(|row| fill_node(&row, theme, cache, viewport.width));

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let list = grid(GRID_ID, &row_nodes, 1, 0, theme.spacing.xs);
    let content_children = [list];
    let content = column(
        CONTENT_ID,
        &content_children,
        theme.spacing.xs,
        Align::Start,
    )
    .with_padding(content_padding(theme));
    let body_children = [crown_node, content];
    let body = column(BODY_ID, &body_children, 0, Align::Start);
    let root_children = [
        crate::view::leaf(BG_ID, Length::Fill(1), Length::Fill(1)),
        body,
    ];
    let root = crate::view::stack(0, &root_children, Align::Start);

    let mut builder = Builder {
        theme,
        cache,
        widgets,
        entries,
        viewport,
    };
    builder.push(background(BG_ID))?;
    builder.push(title)?;
    for row in rows {
        builder.push(row)?;
    }
    builder.layout(&root)
}

/// Builds one page of a category's log list.
pub(super) fn build_list<'a>(
    category: LogCategory,
    entries: &[LogEntry],
    page: usize,
    lines: &'a [[u8; 40]; LOGS_PER_PAGE],
    line_lens: &'a [u8; LOGS_PER_PAGE],
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries_out: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let title = title(category.label());
    let title_node = fill_node(&title, theme, cache, viewport.width);
    let start = page.saturating_mul(LOGS_PER_PAGE);
    let count = entries.len().saturating_sub(start).min(LOGS_PER_PAGE);
    let content_width = viewport.width.saturating_sub(theme.shape.content_inset * 2);

    let mut row_widgets: [Option<Widget<'a>>; LOGS_PER_PAGE] = [None; LOGS_PER_PAGE];
    let mut row_nodes = [spacer(0, Length::Fill(1), Length::Fixed(0)); LOGS_PER_PAGE];
    for index in 0..count {
        let length = usize::from(line_lens[index]).min(lines[index].len());
        let widget = Widget::new(
            ROW_BASE + index as u16,
            WidgetKind::Label(LabelProps {
                text: text(&lines[index][..length]),
                role: TextRole::Caption,
                align: TextAlign::Start,
                icon: None,
            }),
        );
        row_nodes[index] = fill_node(&widget, theme, cache, content_width);
        row_widgets[index] = Some(widget);
    }

    let pages = entries.len().div_ceil(LOGS_PER_PAGE).max(1) as u8;
    let indicator = Widget::new(
        PAGE_ID,
        WidgetKind::PageIndicator(PageIndicatorProps {
            count: pages,
            selected: page.min(usize::from(pages.saturating_sub(1))) as u8,
        }),
    );
    let indicator_node = node(&indicator, theme, cache, Length::Fill(1), content_width);
    let empty = Widget::new(
        EMPTY_ID,
        WidgetKind::Label(LabelProps {
            text: "No logs yet",
            role: TextRole::Body,
            align: TextAlign::Center,
            icon: None,
        }),
    );
    let empty_node = fill_node(&empty, theme, cache, content_width);

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let list = grid(GRID_ID, &row_nodes[..count], 1, 0, theme.spacing.xs);
    let empty_children = [
        spacer(1_031, Length::Fill(1), Length::Fill(1)),
        empty_node,
        spacer(1_032, Length::Fill(1), Length::Fill(1)),
    ];
    let list_children = [
        list,
        spacer(1_033, Length::Fill(1), Length::Fill(1)),
        indicator_node,
    ];
    let content = if count == 0 {
        column(CONTENT_ID, &empty_children, theme.spacing.sm, Align::Center)
            .with_padding(content_padding(theme))
    } else {
        column(CONTENT_ID, &list_children, theme.spacing.sm, Align::Start)
            .with_padding(content_padding(theme))
    };
    let body_children = [crown_node, content];
    let body = column(BODY_ID, &body_children, 0, Align::Start);
    let root_children = [
        crate::view::leaf(BG_ID, Length::Fill(1), Length::Fill(1)),
        body,
    ];
    let root = crate::view::stack(0, &root_children, Align::Start);

    let mut builder = Builder {
        theme,
        cache,
        widgets,
        entries: entries_out,
        viewport,
    };
    builder.push(background(BG_ID))?;
    builder.push(title)?;
    for widget in row_widgets.iter().flatten() {
        builder.push(*widget)?;
    }
    if count == 0 {
        builder.push(empty)?;
    } else {
        builder.push(indicator)?;
    }
    builder.layout(&root)
}

fn categories<'a>() -> [Widget<'a>; LogCategory::ALL.len()] {
    LogCategory::ALL.map(|category| {
        Widget::new(
            category_id(category.index()),
            WidgetKind::SettingsRow(SettingsRowProps {
                title: category.label(),
                subtitle: "",
                icon: Some(icon(category)),
                trailing: SettingsTrailing::Chevron,
                compact: true,
            }),
        )
    })
}

fn icon(category: LogCategory) -> IconId {
    match category {
        LogCategory::Wakes => IconId::Power,
        LogCategory::Alarms => IconId::Bell,
        LogCategory::Battery => IconId::BatteryHigh,
        LogCategory::Modes => IconId::ArrowClockwise,
        LogCategory::Errors => IconId::Warning,
    }
}

fn title<'a>(text: &'a str) -> Widget<'a> {
    Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text,
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    )
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or("")
}
