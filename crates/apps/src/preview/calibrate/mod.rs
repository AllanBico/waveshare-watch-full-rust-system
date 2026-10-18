//! Dev-only panel shape calibration.
//!
//! Draws a candidate rounded-rectangle over the live UI so the radius and
//! optical margin can be matched against the physical panel on hardware.

mod shape;
mod state;

pub(super) use state::{Action, CalibrateState};

use common::Region;
use ui::{
    Align, ButtonProps, IconButtonProps, IconId, Insets, Kind, LabelProps, Length, Node, Painter,
    TextAlign, TextRole, Theme, Widget, WidgetKind,
};

use crate::view::{
    background, column, fill_node, fit_node, leaf, row, spacer, stack, BuildError, Builder,
};

/// Scene id used to damage the shape when it changes.
pub(super) const SCENE_ID: u16 = 600;

const BG_ID: u16 = 500;
pub(super) const BACK_ID: u16 = 501;
const TITLE_ID: u16 = 502;
const READOUT_ID: u16 = 503;

pub(super) const RADIUS_MINUS_5_ID: u16 = 510;
pub(super) const RADIUS_MINUS_1_ID: u16 = 511;
pub(super) const RADIUS_PLUS_1_ID: u16 = 512;
pub(super) const RADIUS_PLUS_5_ID: u16 = 513;
pub(super) const MARGIN_MINUS_5_ID: u16 = 514;
pub(super) const MARGIN_MINUS_1_ID: u16 = 515;
pub(super) const MARGIN_PLUS_1_ID: u16 = 516;
pub(super) const MARGIN_PLUS_5_ID: u16 = 517;
pub(super) const CORNER_ID: u16 = 518;
pub(super) const CLIP_ID: u16 = 519;
pub(super) const RESET_ID: u16 = 520;
const MODE_LABEL_ID: u16 = 521;

const HEADER_ID: u16 = 530;
const RADIUS_ROW_ID: u16 = 531;
const MARGIN_ROW_ID: u16 = 532;
const MODE_ROW_ID: u16 = 533;
const CONTENT_ID: u16 = 534;
const TOP_SPACER_ID: u16 = 535;
const BOTTOM_SPACER_ID: u16 = 536;
const ROOT_ID: u16 = 0;

const PADDING_X: u16 = 32;
const PADDING_Y: u16 = 16;
const SMALL_STEP: i16 = 1;
const LARGE_STEP: i16 = 5;
const OUTLINE_THICKNESS: u16 = 2;

const EMPTY_NODE: Node<'static> =
    Node::container(0, Length::Fixed(0), Length::Fixed(0), Kind::Leaf, &[]);

/// Resolves a widget id into a calibration action.
pub(super) const fn action(id: u16) -> Option<Action> {
    match id {
        RADIUS_MINUS_5_ID => Some(Action::Radius(-LARGE_STEP)),
        RADIUS_MINUS_1_ID => Some(Action::Radius(-SMALL_STEP)),
        RADIUS_PLUS_1_ID => Some(Action::Radius(SMALL_STEP)),
        RADIUS_PLUS_5_ID => Some(Action::Radius(LARGE_STEP)),
        MARGIN_MINUS_5_ID => Some(Action::Margin(-LARGE_STEP)),
        MARGIN_MINUS_1_ID => Some(Action::Margin(-SMALL_STEP)),
        MARGIN_PLUS_1_ID => Some(Action::Margin(SMALL_STEP)),
        MARGIN_PLUS_5_ID => Some(Action::Margin(LARGE_STEP)),
        CORNER_ID => Some(Action::Corner),
        CLIP_ID => Some(Action::Clip),
        RESET_ID => Some(Action::Reset),
        BACK_ID => Some(Action::Back),
        _ => None,
    }
}

/// Builds the calibration screen.
pub(super) fn build<'a>(
    state: &'a CalibrateState,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    let theme = view.theme;
    let content_width = view.viewport.width.saturating_sub(PADDING_X * 2);
    let readout = label(READOUT_ID, state.readout(), TextRole::Caption);
    let mode_label = label(MODE_LABEL_ID, "Corner / Clip", TextRole::Caption);
    let readout_node = fill_node(&readout, theme, view.cache, content_width);
    let mode_label_node = fill_node(&mode_label, theme, view.cache, content_width);

    view.push(background(BG_ID))?;
    view.push(readout)?;
    view.push(mode_label)?;

    let mut header_nodes = [EMPTY_NODE; 2];
    let mut radius_nodes = [EMPTY_NODE; 4];
    let mut margin_nodes = [EMPTY_NODE; 4];
    let mut mode_nodes = [EMPTY_NODE; 3];
    let header = header_row(view, &mut header_nodes, content_width)?;
    let radius_row = radius_row(view, &mut radius_nodes, content_width)?;
    let margin_row = margin_row(view, &mut margin_nodes, content_width)?;
    let mode_row = mode_row(state, view, &mut mode_nodes, content_width)?;

    let parts = Parts {
        header,
        readout: readout_node,
        radius: radius_row,
        margin: margin_row,
        mode_label: mode_label_node,
        mode_row,
    };
    layout_root(view, &parts)
}

struct Parts<'n> {
    header: Node<'n>,
    readout: Node<'n>,
    radius: Node<'n>,
    margin: Node<'n>,
    mode_label: Node<'n>,
    mode_row: Node<'n>,
}

fn layout_root<'a, 'n>(view: &mut Builder<'_, 'a>, parts: &Parts<'n>) -> Result<usize, BuildError> {
    let theme = view.theme;
    let content_children = [
        parts.header,
        spacer(TOP_SPACER_ID, Length::Fill(1), Length::Fill(1)),
        parts.readout,
        parts.radius,
        parts.margin,
        parts.mode_label,
        parts.mode_row,
        spacer(BOTTOM_SPACER_ID, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(
        CONTENT_ID,
        &content_children,
        theme.spacing.sm,
        Align::Start,
    )
    .with_padding(Insets::symmetric(PADDING_X, PADDING_Y));
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), content];
    let root = stack(ROOT_ID, &root_children, Align::Start);
    view.layout(&root)
}

fn header_row<'a, 'n>(
    view: &mut Builder<'_, 'a>,
    nodes: &'n mut [Node<'n>; 2],
    content_width: u16,
) -> Result<Node<'n>, BuildError> {
    let theme = view.theme;
    let back = icon_button(BACK_ID, IconId::CaretLeft, "Back");
    let title = label(TITLE_ID, "Calibration", TextRole::Title);
    nodes[0] = fit_node(&back, theme, view.cache);
    nodes[1] = fill_node(&title, theme, view.cache, content_width);
    view.push(back)?;
    view.push(title)?;
    Ok(row(HEADER_ID, nodes, theme.spacing.sm, Align::Center))
}

fn radius_row<'a, 'n>(
    view: &mut Builder<'_, 'a>,
    nodes: &'n mut [Node<'n>; 4],
    content_width: u16,
) -> Result<Node<'n>, BuildError> {
    let theme = view.theme;
    let widgets = [
        button(RADIUS_MINUS_5_ID, "R-5"),
        button(RADIUS_MINUS_1_ID, "R-1"),
        button(RADIUS_PLUS_1_ID, "R+1"),
        button(RADIUS_PLUS_5_ID, "R+5"),
    ];
    for (slot, widget) in nodes.iter_mut().zip(widgets.iter()) {
        *slot = fill_node(widget, theme, view.cache, content_width);
    }
    for widget in widgets {
        view.push(widget)?;
    }
    Ok(row(RADIUS_ROW_ID, nodes, theme.spacing.sm, Align::Center))
}

fn margin_row<'a, 'n>(
    view: &mut Builder<'_, 'a>,
    nodes: &'n mut [Node<'n>; 4],
    content_width: u16,
) -> Result<Node<'n>, BuildError> {
    let theme = view.theme;
    let widgets = [
        button(MARGIN_MINUS_5_ID, "M-5"),
        button(MARGIN_MINUS_1_ID, "M-1"),
        button(MARGIN_PLUS_1_ID, "M+1"),
        button(MARGIN_PLUS_5_ID, "M+5"),
    ];
    for (slot, widget) in nodes.iter_mut().zip(widgets.iter()) {
        *slot = fill_node(widget, theme, view.cache, content_width);
    }
    for widget in widgets {
        view.push(widget)?;
    }
    Ok(row(MARGIN_ROW_ID, nodes, theme.spacing.sm, Align::Center))
}

fn mode_row<'a, 'n>(
    state: &CalibrateState,
    view: &mut Builder<'_, 'a>,
    nodes: &'n mut [Node<'n>; 3],
    content_width: u16,
) -> Result<Node<'n>, BuildError> {
    let theme = view.theme;
    let widgets = [
        button(CORNER_ID, state.corner_label()),
        button(CLIP_ID, state.clip_label()),
        button(RESET_ID, "Reset"),
    ];
    for (slot, widget) in nodes.iter_mut().zip(widgets.iter()) {
        *slot = fill_node(widget, theme, view.cache, content_width);
    }
    for widget in widgets {
        view.push(widget)?;
    }
    Ok(row(MODE_ROW_ID, nodes, theme.spacing.sm, Align::Center))
}

/// Paints the candidate shape and the inner safe shape.
pub(super) fn paint(
    painter: &mut Painter<'_>,
    state: &CalibrateState,
    theme: &Theme,
    viewport: Region,
) {
    let radii = state.radii();
    if state.clip_ui() {
        shape::mask_outside(painter, viewport, radii, theme.tones.background);
    }
    let margin = state.margin();
    shape::stroke(
        painter,
        shape::inset(viewport, margin),
        shape::reduced(radii, margin),
        theme.tones.text_muted,
    );
    shape::stroke_thick(
        painter,
        viewport,
        radii,
        OUTLINE_THICKNESS,
        theme.tones.text,
    );
}

fn label<'a>(id: u16, text: &'a str, role: TextRole) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::Label(LabelProps {
            text,
            role,
            align: TextAlign::Start,
            icon: None,
        }),
    )
}

fn button(id: u16, text: &'static str) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::Button(ButtonProps {
            label: text,
            icon: None,
        }),
    )
}

fn icon_button(id: u16, icon: IconId, label: &'static str) -> Widget<'static> {
    Widget::new(id, WidgetKind::IconButton(IconButtonProps { icon, label }))
}
