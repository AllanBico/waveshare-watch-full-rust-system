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
    paint_widget, BadgeProps, ButtonProps, CheckboxProps, ConfirmProps, ContextMenuProps,
    DialogProps, EmptyStateProps, ErrorStateProps, IconButtonProps, InputProps, ListItemProps,
    LoadingStateProps, MenuItem, ProgressProps, RadioProps, ScreenScaffoldProps, SliderProps,
    Status, StatusIndicatorProps, TabsProps, ToastProps, ToggleProps, Widget, WidgetKind,
    WidgetState,
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

pub(super) const SAMPLE_KINDS: [WidgetKind<'static>; 20] = [
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
];
