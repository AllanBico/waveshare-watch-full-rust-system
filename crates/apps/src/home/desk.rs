//! Desk Home: a grayscale dashboard built from the same components.

use ui::{
    Align, IconId, Insets, LabelProps, Length, ListItemProps, TextAlign, TextRole, Widget,
    WidgetKind,
};

use super::{
    HomeState, ALARM_ID, BATTERY_ID, BG_ID, CLOCK_ID, DATE_ID, DESK_BATTERY_ID, GRID_ID, HEADER_ID,
    HEADER_SPACER_ID, ROOT_ID, SETTINGS_ID, SYNC_ID, TOP_SPACER_ID,
};
use crate::view::{
    background, column, column_fit, fill_node, fit_node, grid, leaf, row, spacer, BuildError,
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
        TextAlign::Start,
        None,
    );
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
    let charge = if state.is_charging() {
        "Charging"
    } else {
        "On battery"
    };
    let alarm = list(ALARM_ID, "Alarms", "07:00", IconId::Bell);
    let settings = list(SETTINGS_ID, "Settings", "Wi-Fi + BT", IconId::GearSix);
    let battery_tile = list(DESK_BATTERY_ID, "Battery", charge, IconId::BatteryHigh);
    let sync = list(SYNC_ID, "Sync", "Idle", IconId::ArrowClockwise);

    let cell = view.viewport.width.saturating_sub(60) / 2;
    let clock_node = fit_node(&clock, theme, view.cache);
    let date_node = fit_node(&date, theme, view.cache);
    let battery_node = fit_node(&battery, theme, view.cache);
    let alarm_node = fill_node(&alarm, theme, view.cache, cell);
    let settings_node = fill_node(&settings, theme, view.cache, cell);
    let battery_tile_node = fill_node(&battery_tile, theme, view.cache, cell);
    let sync_node = fill_node(&sync, theme, view.cache, cell);
    view.push(background(BG_ID))?;
    for widget in [clock, date, battery, alarm, settings, battery_tile, sync] {
        view.push(widget)?;
    }

    let clock_children = [clock_node, date_node];
    let clock_block = column_fit(10, &clock_children, theme.spacing.xs, Align::Start);
    let top_children = [
        clock_block,
        spacer(HEADER_SPACER_ID, Length::Fill(1), Length::Fixed(0)),
        battery_node,
    ];
    let top = row(HEADER_ID, &top_children, theme.spacing.md, Align::End);
    let grid_children = [alarm_node, settings_node, battery_tile_node, sync_node];
    let dashboard = grid(
        GRID_ID,
        &grid_children,
        2,
        theme.spacing.md,
        theme.spacing.md,
    );
    let content_children = [
        top,
        spacer(
            TOP_SPACER_ID,
            Length::Fill(1),
            Length::Fixed(theme.spacing.lg),
        ),
        dashboard,
        spacer(11, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(ROOT_ID, &content_children, theme.spacing.md, Align::Start)
        .with_padding(Insets::symmetric(24, 24));
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

fn list<'a>(id: u16, title: &'a str, subtitle: &'a str, icon: IconId) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::ListItem(ListItemProps {
            title,
            subtitle,
            leading: Some(icon),
            trailing: None,
        }),
    )
}
