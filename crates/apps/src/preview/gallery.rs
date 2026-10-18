//! A sectioned component gallery for tuning spacing, states, and typography.

mod pages;

use ui::{
    Align, ButtonProps, IconId, LabelProps, Length, PageIndicatorProps, TextAlign, TextRole,
    Widget, WidgetKind,
};

use crate::view::{
    background, column, column_fit, content_padding, crown, fill_node, fit_node, leaf, row, spacer,
    stack, BuildError, Builder,
};

/// Number of gallery pages.
pub const PAGES: u8 = 11;

const BG_ID: u16 = 300;
const TITLE_ID: u16 = 302;
const NEXT_ID: u16 = 303;
const CROWN_ID: u16 = 304;
const CONTENT_ID: u16 = 305;
const BODY_ID: u16 = 306;
const PAGE_DOTS_ID: u16 = 307;
const FOOTER_ID: u16 = 308;
const FOOTER_SPACER_ID: u16 = 309;
const WIDGET_BASE: u16 = 310;
const TOP_SPACER_ID: u16 = 320;
const BOTTOM_SPACER_ID: u16 = 321;

const SECTIONS: [&str; PAGES as usize] = [
    "Actions",
    "Selection",
    "Values",
    "Data",
    "Inputs",
    "Structure",
    "Text",
    "Overlays",
    "States",
    "Error state",
    "Confirm",
];

/// Whether `id` is the next-page control.
pub const fn is_next(id: u16) -> bool {
    id == NEXT_ID
}

pub(super) fn build<'a>(page: u8, view: &mut Builder<'_, 'a>) -> Result<usize, BuildError> {
    let theme = view.theme;
    let page = page % PAGES;
    let title = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: SECTIONS[usize::from(page)],
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    );
    let next = Widget::new(
        NEXT_ID,
        WidgetKind::Button(ButtonProps {
            label: "Next",
            icon: Some(IconId::CaretRight),
        }),
    );
    let dots = Widget::new(
        PAGE_DOTS_ID,
        WidgetKind::PageIndicator(PageIndicatorProps {
            count: PAGES,
            selected: page,
        }),
    );
    let widgets = pages::widgets(page);
    let content_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.content_inset * 2);
    let crown_width = view
        .viewport
        .width
        .saturating_sub(theme.shape.crown_inset * 2);
    let title_node = fill_node(&title, theme, view.cache, crown_width);
    let next_node = fit_node(&next, theme, view.cache);
    let dots_node = fit_node(&dots, theme, view.cache);
    view.push(background(BG_ID))?;
    view.push(title)?;
    view.push(dots)?;
    view.push(next)?;
    for widget in widgets {
        view.push(*widget)?;
    }

    let mut nodes = [ui::Node::spacer(0, Length::Fixed(0), Length::Fixed(0)); pages::MAX_WIDGETS];
    for (index, widget) in widgets.iter().enumerate() {
        nodes[index] = fill_node(widget, theme, view.cache, content_width);
    }
    let items = column_fit(
        BODY_ID,
        &nodes[..widgets.len()],
        theme.spacing.sm,
        Align::Start,
    );
    let footer_children = [
        dots_node,
        spacer(FOOTER_SPACER_ID, Length::Fill(1), Length::Fixed(0)),
        next_node,
    ];
    let footer = row(FOOTER_ID, &footer_children, theme.spacing.sm, Align::Center);
    let content_children = [
        spacer(TOP_SPACER_ID, Length::Fill(1), Length::Fill(1)),
        items,
        spacer(BOTTOM_SPACER_ID, Length::Fill(1), Length::Fill(1)),
        footer,
    ];
    let content =
        column(CONTENT_ID, &content_children, 0, Align::Start).with_padding(content_padding(theme));
    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let body_children = [crown_node, content];
    let body = column(309, &body_children, 0, Align::Start);
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), body];
    let root = stack(0, &root_children, Align::Start);
    view.layout(&root)
}
