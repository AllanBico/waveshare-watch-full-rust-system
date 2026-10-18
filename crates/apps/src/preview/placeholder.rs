//! Placeholder app screens reached from the preview drawer.

use ui::{
    Align, EmptyStateProps, IconButtonProps, IconId, Insets, LabelProps, Length, TextAlign,
    TextRole, Widget, WidgetKind,
};

use crate::view::{background, column, fill_node, fit_node, leaf, row, BuildError, Builder};

const BG_ID: u16 = 400;
const BACK_ID: u16 = 401;
const TITLE_ID: u16 = 402;
const EMPTY_ID: u16 = 403;

/// Whether `id` is the placeholder back control.
pub const fn is_back(id: u16) -> bool {
    id == BACK_ID
}

pub(super) fn build<'a>(
    title: &'static str,
    icon: IconId,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let back = Widget::new(
        BACK_ID,
        WidgetKind::IconButton(IconButtonProps {
            icon: IconId::CaretLeft,
            label: "Back",
        }),
    );
    let heading = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: title,
            role: TextRole::Label,
            align: TextAlign::Start,
            icon: None,
        }),
    );
    let empty = Widget::new(
        EMPTY_ID,
        WidgetKind::EmptyState(EmptyStateProps {
            icon,
            title,
            body: "Demo only. No backend yet.",
        }),
    );
    let content_width = view.viewport.width.saturating_sub(48);
    let back_node = fit_node(&back, theme, view.cache);
    let heading_node = fill_node(&heading, theme, view.cache, content_width);
    view.push(background(BG_ID))?;
    view.push(back)?;
    view.push(heading)?;
    view.push(empty)?;

    let header_children = [back_node, heading_node];
    let header = row(404, &header_children, theme.spacing.sm, Align::Center);
    let content_children = [header, leaf(EMPTY_ID, Length::Fill(1), Length::Fill(1))];
    let content = column(405, &content_children, theme.spacing.sm, Align::Start)
        .with_padding(Insets::symmetric(16, 12));
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), content];
    let root = ui::Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        ui::Kind::Stack {
            align: Align::Start,
        },
        &root_children,
    );
    view.layout(&root)
}
