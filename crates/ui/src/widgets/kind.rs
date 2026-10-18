//! Enum-backed widget kinds and their props.

use super::button::{ButtonProps, IconButtonProps};
use super::card::CardProps;
use super::checkbox::CheckboxProps;
use super::divider::DividerProps;
use super::icon::IconProps;
use super::input::InputProps;
use super::key_value_row::KeyValueRowProps;
use super::label::LabelProps;
use super::list_item::ListItemProps;
use super::overlay::{ContextMenuProps, DialogProps, ToastProps};
use super::page_indicator::PageIndicatorProps;
use super::patterns::{
    ConfirmProps, EmptyStateProps, ErrorStateProps, LoadingStateProps, ScreenScaffoldProps,
};
use super::progress::{BadgeProps, ProgressProps};
use super::radio::RadioProps;
use super::ring::RingProps;
use super::segmented_control::SegmentedControlProps;
use super::settings_row::SettingsRowProps;
use super::slider::SliderProps;
use super::spinner::SpinnerProps;
use super::status::StatusIndicatorProps;
use super::stepper::StepperProps;
use super::tabs::TabsProps;
use super::tile::TileProps;
use super::toggle::ToggleProps;

/// One widget kind with its immutable props.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WidgetKind<'a> {
    /// Draws nothing.
    None,
    /// A labelled action.
    Button(ButtonProps<'a>),
    /// A square icon-only action.
    IconButton(IconButtonProps<'a>),
    /// A standalone icon.
    Icon(IconProps),
    /// One row in a list.
    ListItem(ListItemProps<'a>),
    /// An on/off switch.
    Toggle(ToggleProps<'a>),
    /// An independent checkbox.
    Checkbox(CheckboxProps<'a>),
    /// One choice in a group.
    Radio(RadioProps<'a>),
    /// A continuous value control.
    Slider(SliderProps<'a>),
    /// A text display field.
    Input(InputProps<'a>),
    /// A non-interactive text label.
    Label(LabelProps<'a>),
    /// A value progress bar.
    Progress(ProgressProps<'a>),
    /// A short status label.
    Badge(BadgeProps<'a>),
    /// A modal message body.
    Dialog(DialogProps<'a>),
    /// A transient message.
    Toast(ToastProps<'a>),
    /// A list of menu actions.
    ContextMenu(ContextMenuProps<'a>),
    /// A horizontal tab strip.
    Tabs(TabsProps<'a>),
    /// An icon-and-label tile.
    Tile(TileProps<'a>),
    /// An icon and label status line.
    StatusIndicator(StatusIndicatorProps<'a>),
    /// A titled screen body.
    ScreenScaffold(ScreenScaffoldProps<'a>),
    /// A centered empty-state message.
    EmptyState(EmptyStateProps<'a>),
    /// A centered loading message.
    LoadingState(LoadingStateProps<'a>),
    /// A centered error with one action.
    ErrorState(ErrorStateProps<'a>),
    /// A confirmation prompt.
    Confirm(ConfirmProps<'a>),
    /// A horizontal or vertical rule.
    Divider(DividerProps),
    /// A grouping surface.
    Card(CardProps),
    /// A settings row with a trailing control.
    SettingsRow(SettingsRowProps<'a>),
    /// A small exclusive-choice strip.
    SegmentedControl(SegmentedControlProps<'a>),
    /// A numeric stepper.
    Stepper(StepperProps<'a>),
    /// A circular value ring.
    Ring(RingProps<'a>),
    /// An inline activity spinner.
    Spinner(SpinnerProps),
    /// A label-and-value data row.
    KeyValueRow(KeyValueRowProps<'a>),
    /// A page position indicator.
    PageIndicator(PageIndicatorProps),
}

impl WidgetKind<'_> {
    /// Whether focus traversal may visit this kind.
    pub const fn is_focusable(&self) -> bool {
        matches!(
            self,
            Self::Button(_)
                | Self::IconButton(_)
                | Self::SettingsRow(_)
                | Self::SegmentedControl(_)
                | Self::Stepper(_)
                | Self::ListItem(_)
                | Self::Toggle(_)
                | Self::Checkbox(_)
                | Self::Radio(_)
                | Self::Slider(_)
                | Self::Input(_)
                | Self::ContextMenu(_)
                | Self::Tabs(_)
                | Self::Tile(_)
                | Self::ErrorState(_)
                | Self::Confirm(_)
        )
    }
}
