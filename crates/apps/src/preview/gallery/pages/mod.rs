//! Gallery page catalogs and compact widget constructors.

mod components;
mod display;

use ui::{
    BadgeProps, ButtonProps, CardProps, CardTone, CheckboxProps, ConfirmProps, ContextMenuProps,
    DialogProps, DividerOrientation, DividerProps, EmptyStateProps, ErrorStateProps,
    IconButtonProps, IconId, IconProps, IconSize, InputProps, KeyValueRowProps, LabelProps,
    ListItemProps, LoadingStateProps, MenuItem, ProgressProps, RadioProps, RingProps,
    ScreenScaffoldProps, SegmentedControlProps, SettingsRowProps, SettingsTrailing, SliderProps,
    SpinnerProps, Status, StatusIndicatorProps, StepperProps, TabsProps, TextAlign, TextRole,
    TileProps, ToastProps, ToggleProps, Widget, WidgetKind,
};

/// The largest number of widgets on one page.
pub(super) const MAX_WIDGETS: usize = 5;

const STEPPER_MIN: u16 = 0;
const STEPPER_MAX: u16 = 100;
const STEPPER_STEP: u16 = 1;

/// Returns the widgets for one page.
pub(super) fn widgets(page: u8) -> &'static [Widget<'static>] {
    if page < 6 {
        components::widgets(page)
    } else {
        display::widgets(page)
    }
}

const fn button(id: u16, label: &'static str, icon: Option<IconId>) -> Widget<'static> {
    Widget::new(id, WidgetKind::Button(ButtonProps { label, icon }))
}

const fn icon_button(id: u16, icon: IconId, label: &'static str) -> Widget<'static> {
    Widget::new(id, WidgetKind::IconButton(IconButtonProps { icon, label }))
}

const fn tile(id: u16, label: &'static str, icon: IconId) -> Widget<'static> {
    Widget::new(id, WidgetKind::Tile(TileProps { icon, label }))
}

const fn status(id: u16, label: &'static str, status: Status) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::StatusIndicator(StatusIndicatorProps { label, status }),
    )
}

const fn toggle(id: u16, label: &'static str, checked: bool) -> Widget<'static> {
    Widget::new(id, WidgetKind::Toggle(ToggleProps { checked, label }))
}

const fn checkbox(id: u16, label: &'static str, checked: bool) -> Widget<'static> {
    Widget::new(id, WidgetKind::Checkbox(CheckboxProps { checked, label }))
}

const fn radio(id: u16, label: &'static str, selected: bool) -> Widget<'static> {
    Widget::new(id, WidgetKind::Radio(RadioProps { selected, label }))
}

const fn segmented(id: u16, labels: &'static [&'static str], selected: u8) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::SegmentedControl(SegmentedControlProps { labels, selected }),
    )
}

const fn slider(id: u16, label: &'static str, value: u8) -> Widget<'static> {
    Widget::new(id, WidgetKind::Slider(SliderProps { value, label }))
}

const fn tabs(id: u16, labels: &'static [&'static str], selected: u8) -> Widget<'static> {
    Widget::new(id, WidgetKind::Tabs(TabsProps { labels, selected }))
}

const fn stepper(id: u16, label: &'static str, value: u16, unit: &'static str) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::Stepper(StepperProps {
            label,
            value,
            min: STEPPER_MIN,
            max: STEPPER_MAX,
            step: STEPPER_STEP,
            unit,
        }),
    )
}

const fn progress(id: u16, label: &'static str, value: u8) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::Progress(ProgressProps {
            value,
            label,
            indeterminate: false,
        }),
    )
}

const fn spinner(id: u16) -> Widget<'static> {
    Widget::new(id, WidgetKind::Spinner(SpinnerProps { phase: 2 }))
}

const fn ring(id: u16, label: &'static str, value: u8) -> Widget<'static> {
    Widget::new(id, WidgetKind::Ring(RingProps { value, label }))
}

const fn key_value(id: u16, key: &'static str, value: &'static str) -> Widget<'static> {
    Widget::new(id, WidgetKind::KeyValueRow(KeyValueRowProps { key, value }))
}

const fn icon(id: u16, icon: IconId, size: IconSize) -> Widget<'static> {
    Widget::new(id, WidgetKind::Icon(IconProps { icon, size }))
}

const fn divider(id: u16, orientation: DividerOrientation) -> Widget<'static> {
    Widget::new(id, WidgetKind::Divider(DividerProps { orientation }))
}

const fn input(id: u16, value: &'static str, caret: Option<u16>) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::Input(InputProps {
            value,
            placeholder: "Name",
            masked: false,
            caret,
        }),
    )
}

const fn settings_row(
    id: u16,
    title: &'static str,
    subtitle: &'static str,
    icon: Option<IconId>,
    trailing: SettingsTrailing<'static>,
) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::SettingsRow(SettingsRowProps {
            title,
            subtitle,
            icon,
            trailing,
        }),
    )
}

const fn list_item(
    id: u16,
    title: &'static str,
    subtitle: &'static str,
    leading: Option<IconId>,
) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::ListItem(ListItemProps {
            title,
            subtitle,
            leading,
            trailing: Some(IconId::CaretRight),
        }),
    )
}

const fn card(id: u16, tone: CardTone) -> Widget<'static> {
    Widget::new(id, WidgetKind::Card(CardProps { tone }))
}

const fn label(id: u16, text: &'static str, role: TextRole, align: TextAlign) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::Label(LabelProps {
            text,
            role,
            align,
            icon: None,
        }),
    )
}

const fn badge(id: u16, text: &'static str) -> Widget<'static> {
    Widget::new(id, WidgetKind::Badge(BadgeProps { label: text }))
}

const fn scaffold(id: u16, title: &'static str, body: &'static str) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::ScreenScaffold(ScreenScaffoldProps { title, body }),
    )
}

const fn dialog(id: u16, title: &'static str, body: &'static str) -> Widget<'static> {
    Widget::new(id, WidgetKind::Dialog(DialogProps { title, body }))
}

const fn toast(id: u16, message: &'static str, icon: Option<IconId>) -> Widget<'static> {
    Widget::new(id, WidgetKind::Toast(ToastProps { message, icon }))
}

const fn menu(id: u16, items: &'static [MenuItem<'static>]) -> Widget<'static> {
    Widget::new(id, WidgetKind::ContextMenu(ContextMenuProps { items }))
}

const fn empty(id: u16, icon: IconId, title: &'static str, body: &'static str) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::EmptyState(EmptyStateProps { icon, title, body }),
    )
}

const fn loading(id: u16, label: &'static str) -> Widget<'static> {
    Widget::new(id, WidgetKind::LoadingState(LoadingStateProps { label }))
}

const fn error(
    id: u16,
    title: &'static str,
    body: &'static str,
    retry: &'static str,
) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::ErrorState(ErrorStateProps { title, body, retry }),
    )
}

const fn confirm(
    id: u16,
    title: &'static str,
    body: &'static str,
    confirm: &'static str,
    cancel: &'static str,
) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::Confirm(ConfirmProps {
            title,
            body,
            confirm,
            cancel,
        }),
    )
}
