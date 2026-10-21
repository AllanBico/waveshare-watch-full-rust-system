//! Desk Home: a grayscale clock with live system stats.

use ui::{
    Align, KeyValueRowProps, Kind, LabelProps, Length, Node, TextAlign, TextRole, Widget,
    WidgetKind,
};

use super::{
    HomeState, BATTERY_ID, BG_ID, CLOCK_ID, CONTENT_ID, CROWN_ID, DATE_ID, HEADER_ID,
    HEADER_SPACER_ID, ROOT_ID, STATS_CPU1_ID, STATS_CPU_ID, STATS_ID, STATS_PSRAM_ID, STATS_RAM_ID,
    STATS_SPEED_ID,
};
use crate::view::{
    background, column, content_padding, crown, fill_node, fit_node, leaf, row, spacer, BuildError,
    Builder,
};

pub(super) fn build<'a>(
    state: &'a HomeState,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let clock = label(
        CLOCK_ID,
        state.clock(),
        TextRole::Numeric,
        TextAlign::Center,
        None,
    );
    let date = label(
        DATE_ID,
        state.date(),
        TextRole::Caption,
        TextAlign::Start,
        None,
    );
    let battery_icon = if state.is_charging() {
        ui::IconId::Usb
    } else {
        ui::IconId::BatteryHigh
    };
    let battery = label(
        BATTERY_ID,
        state.battery(),
        TextRole::Caption,
        TextAlign::End,
        Some(battery_icon),
    );
    let cpu = key_value(STATS_CPU_ID, "C0", state.cpu());
    let cpu1 = key_value(STATS_CPU1_ID, "C1", state.cpu1());
    let ram = key_value(STATS_RAM_ID, "RAM", state.ram());
    let psram = key_value(STATS_PSRAM_ID, "PSRAM", state.psram());
    let speed = key_value(STATS_SPEED_ID, "SPD", state.speed());

    let content_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.content_inset * 2);
    let clock_node = fill_node(&clock, theme, view.cache, content_width);
    let date_node = fit_node(&date, theme, view.cache);
    let battery_node = fit_node(&battery, theme, view.cache);
    let cpu_node = fill_node(&cpu, theme, view.cache, content_width);
    let cpu1_node = fill_node(&cpu1, theme, view.cache, content_width);
    let ram_node = fill_node(&ram, theme, view.cache, content_width);
    let psram_node = fill_node(&psram, theme, view.cache, content_width);
    let speed_node = fill_node(&speed, theme, view.cache, content_width);
    view.push(background(BG_ID))?;
    for widget in [clock, date, battery, cpu, cpu1, ram, psram, speed] {
        view.push(widget)?;
    }

    let header_children = [
        date_node,
        spacer(HEADER_SPACER_ID, Length::Fill(1), Length::Fixed(0)),
        battery_node,
    ];
    let header = row(HEADER_ID, &header_children, theme.spacing.sm, Align::Center);
    let crown_children = [header];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let stats_children = [cpu_node, cpu1_node, ram_node, psram_node, speed_node];
    let stats = Node::container(
        STATS_ID,
        Length::Fill(1),
        Length::Fit,
        Kind::Column {
            spacing: theme.spacing.xs,
            cross_align: Align::Start,
        },
        &stats_children,
    );
    let content_children = [
        spacer(12, Length::Fill(1), Length::Fill(1)),
        clock_node,
        spacer(13, Length::Fill(1), Length::Fixed(theme.spacing.md)),
        stats,
        spacer(14, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(
        CONTENT_ID,
        &content_children,
        theme.spacing.sm,
        Align::Start,
    )
    .with_padding(content_padding(theme));
    let body_children = [crown_node, content];
    let body = column(ROOT_ID, &body_children, 0, Align::Start);
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), body];
    let root = crate::view::stack(0, &root_children, Align::Start);
    view.layout(&root)
}

fn key_value<'a>(id: u16, key: &'a str, value: &'a str) -> Widget<'a> {
    Widget::new(id, WidgetKind::KeyValueRow(KeyValueRowProps { key, value }))
}

fn label<'a>(
    id: u16,
    text: &'a str,
    role: TextRole,
    align: TextAlign,
    icon: Option<ui::IconId>,
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
