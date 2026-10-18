//! Enum-backed widget kinds and their props.

use super::button::{ButtonProps, IconButtonProps};
use super::checkbox::CheckboxProps;
use super::input::InputProps;
use super::list_item::ListItemProps;
use super::overlay::{ContextMenuProps, DialogProps, ToastProps};
use super::patterns::{
    ConfirmProps, EmptyStateProps, ErrorStateProps, LoadingStateProps, ScreenScaffoldProps,
};
use super::progress::{BadgeProps, ProgressProps};
use super::radio::RadioProps;
use super::slider::SliderProps;
use super::status::StatusIndicatorProps;
use super::tabs::TabsProps;
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
}

impl WidgetKind<'_> {
    /// Whether focus traversal may visit this kind.
    pub const fn is_focusable(&self) -> bool {
        matches!(
            self,
            Self::Button(_)
                | Self::IconButton(_)
                | Self::ListItem(_)
                | Self::Toggle(_)
                | Self::Checkbox(_)
                | Self::Radio(_)
                | Self::Slider(_)
                | Self::Input(_)
                | Self::ContextMenu(_)
                | Self::Tabs(_)
                | Self::ErrorState(_)
                | Self::Confirm(_)
        )
    }
}
