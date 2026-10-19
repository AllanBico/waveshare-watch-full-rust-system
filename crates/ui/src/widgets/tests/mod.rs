//! Widget tests.

mod budget;
mod focus;
mod golden;
mod states;

use common::Region;

use crate::assets::IconId;
use crate::painter::Painter;
use crate::surface::{GraySurface, MonoSurface};
use crate::test_support::{fnv1a, gray_ppm, mono_ppm};
use crate::theme::Theme;
use crate::widgets::{
    paint_widget, BadgeProps, ButtonProps, CardProps, CardTone, CheckboxProps, ConfirmProps,
    ContextMenuProps, DialogProps, DividerOrientation, DividerProps, EmptyStateProps,
    ErrorStateProps, IconButtonProps, IconProps, IconSize, InputProps, KeyValueRowProps,
    ListItemProps, LoadingStateProps, MenuItem, PageIndicatorProps, ProgressProps, RadioProps,
    RingProps, ScreenScaffoldProps, SegmentedControlProps, SettingsRowProps, SettingsTrailing,
    SliderProps, SpinnerProps, Status, StatusIndicatorProps, StepperProps, TabsProps, ToastProps,
    ToggleProps, Widget, WidgetKind, WidgetState,
};

fn render_watch(kind: WidgetKind<'_>, state: WidgetState, width: u16, height: u16) -> (u64, u32) {
    let mut data = vec![0u8; MonoSurface::buffer_len(width, height)];
    let mut surface = MonoSurface::new(width, height, &mut data).unwrap();
    {
        let mut painter = Painter::new(&mut surface);
        let widget = Widget::new(1, kind).with_state(state);
        paint_widget(
            &mut painter,
            &widget,
            Region::full(width, height),
            &Theme::WATCH,
        );
    }
    (fnv1a(&mono_ppm(&surface)), surface.lit_pixels())
}

fn render_desk(kind: WidgetKind<'_>, state: WidgetState, width: u16, height: u16) -> u64 {
    let mut data = vec![0u8; GraySurface::buffer_len(width, height)];
    let mut surface = GraySurface::new(width, height, &mut data).unwrap();
    {
        let mut painter = Painter::new(&mut surface);
        let widget = Widget::new(1, kind).with_state(state);
        paint_widget(
            &mut painter,
            &widget,
            Region::full(width, height),
            &Theme::DESK,
        );
    }
    fnv1a(&gray_ppm(&surface))
}

pub(super) const SAMPLE_KINDS: [WidgetKind<'static>; 30] = [
    WidgetKind::Button(ButtonProps {
        label: "Continue",
        icon: None,
    }),
    WidgetKind::IconButton(IconButtonProps {
        icon: IconId::GearSix,
        label: "Settings",
    }),
    WidgetKind::ListItem(ListItemProps {
        title: "Bluetooth",
        subtitle: "Paired",
        leading: Some(IconId::Bluetooth),
        trailing: Some(IconId::CaretRight),
    }),
    WidgetKind::Toggle(ToggleProps {
        checked: true,
        label: "Wi-Fi",
    }),
    WidgetKind::Checkbox(CheckboxProps {
        checked: true,
        label: "Sync",
    }),
    WidgetKind::Radio(RadioProps {
        selected: true,
        label: "Option A",
    }),
    WidgetKind::Slider(SliderProps {
        value: 60,
        label: "Volume",
    }),
    WidgetKind::Input(InputProps {
        value: "hello",
        placeholder: "Name",
        masked: false,
        caret: Some(5),
    }),
    WidgetKind::Progress(ProgressProps {
        value: 60,
        label: "Sync",
        indeterminate: false,
    }),
    WidgetKind::Badge(BadgeProps { label: "New" }),
    WidgetKind::Dialog(DialogProps {
        title: "Delete?",
        body: "This cannot be undone.",
    }),
    WidgetKind::Toast(ToastProps {
        message: "Saved",
        icon: Some(IconId::CheckCircle),
    }),
    WidgetKind::ContextMenu(ContextMenuProps {
        items: &[
            MenuItem {
                label: "Open",
                icon: Some(IconId::CaretRight),
            },
            MenuItem {
                label: "Delete",
                icon: Some(IconId::Trash),
            },
        ],
    }),
    WidgetKind::Tabs(TabsProps {
        labels: &["Home", "Apps", "Settings"],
        selected: 1,
    }),
    WidgetKind::StatusIndicator(StatusIndicatorProps {
        label: "Connected",
        status: Status::Ok,
    }),
    WidgetKind::ScreenScaffold(ScreenScaffoldProps {
        title: "Settings",
        body: "Manage your watch.",
    }),
    WidgetKind::EmptyState(EmptyStateProps {
        icon: IconId::Bell,
        title: "No alarms",
        body: "Add an alarm to get started.",
    }),
    WidgetKind::LoadingState(LoadingStateProps { label: "Loading" }),
    WidgetKind::ErrorState(ErrorStateProps {
        title: "Sync failed",
        body: "Check your connection.",
        retry: "Retry",
    }),
    WidgetKind::Confirm(ConfirmProps {
        title: "Delete?",
        body: "This cannot be undone.",
        confirm: "Delete",
        cancel: "Cancel",
    }),
    WidgetKind::Divider(DividerProps {
        orientation: DividerOrientation::Horizontal,
    }),
    WidgetKind::Card(CardProps {
        tone: CardTone::Surface,
    }),
    WidgetKind::Icon(IconProps {
        icon: IconId::Heart,
        size: IconSize::Lg,
    }),
    WidgetKind::SettingsRow(SettingsRowProps {
        title: "Bluetooth",
        subtitle: "Paired",
        icon: Some(IconId::Bluetooth),
        trailing: SettingsTrailing::Toggle(true),
        compact: false,
    }),
    WidgetKind::SegmentedControl(SegmentedControlProps {
        labels: &["Day", "Week", "Month"],
        selected: 1,
    }),
    WidgetKind::Stepper(StepperProps {
        label: "Alarm",
        value: 7,
        min: 0,
        max: 23,
        step: 1,
        unit: "h",
    }),
    WidgetKind::Ring(RingProps {
        value: 60,
        label: "Battery",
    }),
    WidgetKind::Spinner(SpinnerProps { phase: 2 }),
    WidgetKind::KeyValueRow(KeyValueRowProps {
        key: "Steps",
        value: "8420",
    }),
    WidgetKind::PageIndicator(PageIndicatorProps {
        count: 5,
        selected: 2,
    }),
];
