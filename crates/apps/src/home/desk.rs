//! Desk Home: a grayscale dashboard built from the same components.

use ui::{
    Align, IconId, LabelProps, Length, ListItemProps, TextAlign, TextRole, Widget, WidgetKind,
};

use super::{
    HomeState, ALARM_ID, BG_ID, CLOCK_ID, CONTENT_ID, CROWN_ID, DATE_ID, DESK_BATTERY_ID, GRID_ID,
    ROOT_ID, SETTINGS_ID, SYNC_ID,
};
use crate::view::{
    background, column, content_padding, crown, fill_node, grid, leaf, spacer, BuildError, Builder,
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
        TextAlign::Center,
        None,
    );
    let charge = if state.is_charging() {
        "Charging"
    } else {
        "On battery"
    };
    let alarm = list(ALARM_ID, "Alarms", "07:00", IconId::Bell);
    let settings = list(SETTINGS_ID, "Settings", "Wi-Fi + BT", IconId::GearSix);
    let battery = list(DESK_BATTERY_ID, "Battery", charge, IconId::BatteryHigh);
    let sync = list(SYNC_ID, "Sync", "Idle", IconId::ArrowClockwise);

    let content_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.content_inset * 2);
    let cell = content_width.saturating_sub(theme.spacing.sm) / 2;
    let clock_node = fill_node(&clock, theme, view.cache, content_width);
    let date_node = fill_node(&date, theme, view.cache, content_width);
    let alarm_node = fill_node(&alarm, theme, view.cache, cell);
    let settings_node = fill_node(&settings, theme, view.cache, cell);
    let battery_node = fill_node(&battery, theme, view.cache, cell);
    let sync_node = fill_node(&sync, theme, view.cache, cell);
    view.push(background(BG_ID))?;
    for widget in [clock, date, alarm, settings, battery, sync] {
        view.push(widget)?;
    }

    let crown_children = [date_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let grid_children = [alarm_node, settings_node, battery_node, sync_node];
    let dashboard = grid(
        GRID_ID,
        &grid_children,
        2,
        theme.spacing.sm,
        theme.spacing.sm,
    );
    let content_children = [
        spacer(12, Length::Fill(1), Length::Fill(1)),
        clock_node,
        spacer(13, Length::Fill(1), Length::Fixed(theme.spacing.md)),
        dashboard,
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
