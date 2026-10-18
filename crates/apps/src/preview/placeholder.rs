//! Placeholder app screens reached from the preview drawer.

use ui::{
    Align, EmptyStateProps, IconId, LabelProps, Length, TextAlign, TextRole, Widget, WidgetKind,
};

use crate::view::{
    background, column, content_padding, crown, fill_node, leaf, spacer, stack, BuildError, Builder,
};

const BG_ID: u16 = 400;
const TITLE_ID: u16 = 402;
const EMPTY_ID: u16 = 403;
const CROWN_ID: u16 = 406;
const CONTENT_ID: u16 = 407;
const BODY_ID: u16 = 410;

pub(super) fn build<'a>(
    title: &'static str,
    icon: IconId,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let heading = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: title,
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    );
    let empty = Widget::new(
        EMPTY_ID,
        WidgetKind::EmptyState(EmptyStateProps {
            icon,
            title,
            body: "Swipe from left to go back.",
        }),
    );
    let content_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.crown_inset * 2);
    let heading_node = fill_node(&heading, theme, view.cache, content_width);
    let empty_node = leaf(EMPTY_ID, Length::Fill(1), Length::Fill(1));
    view.push(background(BG_ID))?;
    view.push(heading)?;
    view.push(empty)?;

    let crown_children = [heading_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let content_children = [
        spacer(408, Length::Fill(1), Length::Fill(1)),
        empty_node,
        spacer(409, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(CONTENT_ID, &content_children, 0, Align::Center)
        .with_padding(content_padding(theme));
    let body_children = [crown_node, content];
    let body = column(BODY_ID, &body_children, 0, Align::Start);
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), body];
    let root = stack(0, &root_children, Align::Start);
    view.layout(&root)
}
