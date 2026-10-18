//! Widget measurement and layout nodes.

use crate::layout::{Kind, Length, Node, Size};
use crate::text::TextCache;
use crate::theme::Theme;

use super::button;
use super::card;
use super::checkbox;
use super::divider;
use super::icon;
use super::input;
use super::key_value_row;
use super::label;
use super::list_item;
use super::overlay;
use super::page_indicator;
use super::patterns;
use super::progress;
use super::radio;
use super::ring;
use super::segmented_control;
use super::settings_row;
use super::slider;
use super::spinner;
use super::status;
use super::stepper;
use super::tabs;
use super::tile;
use super::toggle;
use super::{Widget, WidgetKind};

impl<'a> Widget<'a> {
    /// Measures the widget for the given available width.
    pub fn measure(&self, theme: &Theme, cache: &mut TextCache, available_width: u16) -> Size {
        match &self.kind {
            WidgetKind::None => Size::ZERO,
            WidgetKind::Button(_) | WidgetKind::IconButton(_) => {
                button::measure(&self.kind, theme, cache, available_width)
            }
            WidgetKind::Icon(props) => icon::measure(props, theme, cache, available_width),
            WidgetKind::ListItem(props) => list_item::measure(props, theme, cache, available_width),
            WidgetKind::Toggle(props) => toggle::measure(props, theme, cache, available_width),
            WidgetKind::Checkbox(props) => checkbox::measure(props, theme, cache, available_width),
            WidgetKind::Radio(props) => radio::measure(props, theme, cache, available_width),
            WidgetKind::Slider(props) => slider::measure(props, theme, cache, available_width),
            WidgetKind::Input(props) => input::measure(props, theme, cache, available_width),
            WidgetKind::Label(props) => label::measure(props, theme, cache, available_width),
            WidgetKind::Progress(_) | WidgetKind::Badge(_) => {
                progress::measure(&self.kind, theme, cache, available_width)
            }
            WidgetKind::Dialog(_) | WidgetKind::Toast(_) | WidgetKind::ContextMenu(_) => {
                overlay::measure(&self.kind, theme, cache, available_width)
            }
            WidgetKind::Tabs(props) => tabs::measure(props, theme, cache, available_width),
            WidgetKind::Tile(props) => tile::measure(props, theme, cache, available_width),
            WidgetKind::StatusIndicator(props) => {
                status::measure(props, theme, cache, available_width)
            }
            WidgetKind::ScreenScaffold(_)
            | WidgetKind::EmptyState(_)
            | WidgetKind::LoadingState(_)
            | WidgetKind::ErrorState(_)
            | WidgetKind::Confirm(_) => {
                patterns::measure(&self.kind, theme, cache, available_width)
            }
            WidgetKind::Divider(props) => divider::measure(props, theme, cache, available_width),
            WidgetKind::Card(props) => card::measure(props, theme, cache, available_width),
            WidgetKind::SettingsRow(props) => {
                settings_row::measure(props, theme, cache, available_width)
            }
            WidgetKind::SegmentedControl(props) => {
                segmented_control::measure(props, theme, cache, available_width)
            }
            WidgetKind::Stepper(props) => stepper::measure(props, theme, cache, available_width),
            WidgetKind::Ring(props) => ring::measure(props, theme, cache, available_width),
            WidgetKind::Spinner(props) => spinner::measure(props, theme, cache, available_width),
            WidgetKind::KeyValueRow(props) => {
                key_value_row::measure(props, theme, cache, available_width)
            }
            WidgetKind::PageIndicator(props) => {
                page_indicator::measure(props, theme, cache, available_width)
            }
        }
    }

    /// Builds a leaf layout node carrying this widget's stable id.
    pub fn node(
        &self,
        theme: &Theme,
        cache: &mut TextCache,
        available_width: u16,
    ) -> Node<'static> {
        let size = self.measure(theme, cache, available_width);
        Node::container(
            self.id,
            Length::Fixed(size.width),
            Length::Fixed(size.height),
            Kind::Leaf,
            &[],
        )
    }
}
