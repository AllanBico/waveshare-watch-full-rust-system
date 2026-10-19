//! Desk Home: a grayscale clock with a desktop mode badge.

use ui::{
    Align, IconId, IconProps, IconSize, Kind, LabelProps, Length, Node, TextAlign, TextRole,
    Widget, WidgetKind,
};

use super::{
    HomeState, BATTERY_ID, BG_ID, CLOCK_ID, CONTENT_ID, CROWN_ID, DATE_ID, DESKTOP_ICON_ID,
    DESKTOP_ID, DESKTOP_LABEL_ID, HEADER_ID, HEADER_SPACER_ID, ROOT_ID,
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
        IconId::Usb
    } else {
        IconId::BatteryHigh
    };
    let battery = label(
        BATTERY_ID,
        state.battery(),
        TextRole::Caption,
        TextAlign::End,
        Some(battery_icon),
    );
    let badge_icon = Widget::new(
        DESKTOP_ICON_ID,
        WidgetKind::Icon(IconProps {
            icon: IconId::Desktop,
            size: IconSize::Xl,
        }),
    );
    let badge_label = label(
        DESKTOP_LABEL_ID,
        "Desktop",
        TextRole::Label,
        TextAlign::Center,
        None,
    );

    let content_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.content_inset * 2);
    let clock_node = fill_node(&clock, theme, view.cache, content_width);
    let date_node = fit_node(&date, theme, view.cache);
    let battery_node = fit_node(&battery, theme, view.cache);
    let badge_icon_node = fit_node(&badge_icon, theme, view.cache);
    let badge_label_node = fill_node(&badge_label, theme, view.cache, content_width);
    view.push(background(BG_ID))?;
    for widget in [clock, date, battery, badge_icon, badge_label] {
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
    let badge_children = [badge_icon_node, badge_label_node];
    let badge = Node::container(
        DESKTOP_ID,
        Length::Fill(1),
        Length::Fit,
        Kind::Column {
            spacing: theme.spacing.xs,
            cross_align: Align::Center,
        },
        &badge_children,
    );
    let content_children = [
        spacer(12, Length::Fill(1), Length::Fill(1)),
        clock_node,
        spacer(13, Length::Fill(1), Length::Fixed(theme.spacing.md)),
        badge,
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
