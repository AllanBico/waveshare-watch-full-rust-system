//! Watch Home: a light clock, date, battery, and minimal chrome.

use ui::{Align, IconId, Insets, LabelProps, Length, TextAlign, TextRole, Widget, WidgetKind};

use super::{
    HomeState, BATTERY_ID, BG_ID, BOTTOM_SPACER_ID, CLOCK_ID, DATE_ID, HEADER_ID, HEADER_SPACER_ID,
    ROOT_ID, TOP_SPACER_ID,
};
use crate::view::{
    background, column, fill_node, fit_node, leaf, row, spacer, BuildError, Builder,
};

pub(super) fn build<'a>(
    state: &'a HomeState,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let date = label(
        DATE_ID,
        state.date(),
        TextRole::Caption,
        TextAlign::Start,
        None,
    );
    let battery = label(
        BATTERY_ID,
        state.battery(),
        TextRole::Caption,
        TextAlign::End,
        Some(IconId::BatteryHigh),
    );
    let clock = label(
        CLOCK_ID,
        state.clock(),
        TextRole::Numeric,
        TextAlign::Center,
        None,
    );
    let date_node = fit_node(&date, theme, view.cache);
    let battery_node = fit_node(&battery, theme, view.cache);
    let clock_node = fill_node(
        &clock,
        theme,
        view.cache,
        view.viewport.width.saturating_sub(40),
    );
    view.push(background(BG_ID))?;
    view.push(date)?;
    view.push(battery)?;
    view.push(clock)?;

    let header_children = [
        date_node,
        spacer(HEADER_SPACER_ID, Length::Fill(1), Length::Fixed(0)),
        battery_node,
    ];
    let header = row(HEADER_ID, &header_children, theme.spacing.sm, Align::Center);
    let content_children = [
        header,
        spacer(TOP_SPACER_ID, Length::Fill(1), Length::Fill(1)),
        clock_node,
        spacer(BOTTOM_SPACER_ID, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(ROOT_ID, &content_children, theme.spacing.sm, Align::Center)
        .with_padding(Insets::symmetric(20, 16));
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), content];
    let root = crate::view::stack(0, &root_children, Align::Start);
    view.layout(&root)
}

fn label<'a>(
    id: u16,
    text: &'a str,
    role: TextRole,
    align: TextAlign,
    icon: Option<IconId>,
) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::Label(LabelProps {
            text,
            role,
            align,
            icon,
        }),
    )
}
