//! Widget measurement and layout nodes.

use crate::layout::{Kind, Length, Node, Size};
use crate::text::TextCache;
use crate::theme::Theme;

use super::button;
use super::checkbox;
use super::input;
use super::list_item;
use super::overlay;
use super::patterns;
use super::progress;
use super::radio;
use super::slider;
use super::status;
use super::tabs;
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
            WidgetKind::ListItem(props) => list_item::measure(props, theme, cache, available_width),
            WidgetKind::Toggle(props) => toggle::measure(props, theme, cache, available_width),
            WidgetKind::Checkbox(props) => checkbox::measure(props, theme, cache, available_width),
            WidgetKind::Radio(props) => radio::measure(props, theme, cache, available_width),
            WidgetKind::Slider(props) => slider::measure(props, theme, cache, available_width),
            WidgetKind::Input(props) => input::measure(props, theme, cache, available_width),
            WidgetKind::Progress(_) | WidgetKind::Badge(_) => {
                progress::measure(&self.kind, theme, cache, available_width)
            }
            WidgetKind::Dialog(_) | WidgetKind::Toast(_) | WidgetKind::ContextMenu(_) => {
                overlay::measure(&self.kind, theme, cache, available_width)
            }
            WidgetKind::Tabs(props) => tabs::measure(props, theme, cache, available_width),
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
