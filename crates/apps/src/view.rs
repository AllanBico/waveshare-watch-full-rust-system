//! Fixed-capacity view construction shared by the screens.

use common::Region;
use ui::{
    layout, Align, Insets, Kind, LayoutEntry, LayoutError, Length, Node, NodeId,
    ScreenScaffoldProps, TextCache, Theme, Widget, WidgetArena, WidgetError, WidgetKind,
};

/// Maximum layout entries retained by one screen.
pub(crate) const ENTRY_CAPACITY: usize = 128;

const EMPTY_ENTRY: LayoutEntry = LayoutEntry {
    id: 0,
    rect: ui::LayoutRect::new(0, 0, 0, 0),
    clip: Region::new(0, 0, 0, 0),
};

/// A view construction failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuildError {
    /// The widget arena is full.
    Widget(WidgetError),
    /// Layout failed.
    Layout(LayoutError),
}

/// Borrowed state for building one frame.
pub(crate) struct Builder<'a, 'w> {
    /// Active theme.
    pub theme: &'a Theme,
    /// Shared text measurement cache.
    pub cache: &'a mut TextCache,
    /// Widget arena for this frame.
    pub widgets: &'a mut WidgetArena<'w>,
    /// Layout entry output.
    pub entries: &'a mut [LayoutEntry],
    /// Screen rectangle.
    pub viewport: Region,
}

impl<'a, 'w> Builder<'a, 'w> {
    /// Pushes a widget into the frame arena.
    pub(crate) fn push(&mut self, widget: Widget<'w>) -> Result<(), BuildError> {
        self.widgets.push(widget).map_err(BuildError::Widget)
    }

    /// Lays out the frame tree and returns the entry count.
    pub(crate) fn layout(&mut self, root: &Node<'_>) -> Result<usize, BuildError> {
        run_layout(root, self.viewport, self.cache, self.entries)
    }
}

/// An empty layout entry array.
pub(crate) const fn entries() -> [LayoutEntry; ENTRY_CAPACITY] {
    [EMPTY_ENTRY; ENTRY_CAPACITY]
}

/// The full-screen background used by every screen.
pub(crate) fn background(id: NodeId) -> Widget<'static> {
    Widget::new(
        id,
        WidgetKind::ScreenScaffold(ScreenScaffoldProps {
            title: "",
            body: "",
        }),
    )
}

/// A leaf node for a widget with a measured height and the given width.
pub(crate) fn node(
    widget: &Widget<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    width: Length,
    available: u16,
) -> Node<'static> {
    let size = widget.measure(theme, cache, available);
    Node::container(
        widget.id,
        width,
        Length::Fixed(size.height),
        Kind::Leaf,
        &[],
    )
}

/// A leaf node sized to the widget's intrinsic size.
pub(crate) fn fit_node(widget: &Widget<'_>, theme: &Theme, cache: &mut TextCache) -> Node<'static> {
    let size = widget.measure(theme, cache, ui::UNBOUNDED);
    Node::container(
        widget.id,
        Length::Fixed(size.width),
        Length::Fixed(size.height),
        Kind::Leaf,
        &[],
    )
}

/// A leaf node for a widget with an explicit height.
pub(crate) fn sized_node(id: NodeId, width: Length, height: u16) -> Node<'static> {
    Node::container(id, width, Length::Fixed(height), Kind::Leaf, &[])
}

/// A full-width leaf node for a widget.
pub(crate) fn fill_node(
    widget: &Widget<'_>,
    theme: &Theme,
    cache: &mut TextCache,
    available: u16,
) -> Node<'static> {
    node(widget, theme, cache, Length::Fill(1), available)
}

/// Lays out the tree into `entries`.
pub(crate) fn run_layout(
    root: &Node<'_>,
    viewport: Region,
    cache: &mut TextCache,
    entries: &mut [LayoutEntry],
) -> Result<usize, BuildError> {
    layout(root, viewport, Insets::ZERO, cache, entries).map_err(BuildError::Layout)
}

/// A fixed-height crown at the top of the panel.
pub(crate) fn crown<'a>(id: NodeId, children: &'a [Node<'a>], theme: &Theme) -> Node<'a> {
    Node::container(
        id,
        Length::Fill(1),
        Length::Fixed(theme.shape.vertical_inset()),
        Kind::Stack {
            align: Align::Center,
        },
        children,
    )
    .with_padding(theme.shape.crown_insets())
}

/// Padding for content below the crown.
pub(crate) fn content_padding(theme: &Theme) -> Insets {
    let shape = theme.shape;
    Insets::new(
        0,
        shape.content_inset,
        shape.bottom_inset(),
        shape.content_inset,
    )
}

/// A flexible spacer.
pub(crate) fn spacer(id: NodeId, width: Length, height: Length) -> Node<'static> {
    Node::spacer(id, width, height)
}

/// A row node over caller-owned children.
pub(crate) fn row<'a>(
    id: NodeId,
    children: &'a [Node<'a>],
    spacing: u16,
    align: Align,
) -> Node<'a> {
    Node::container(
        id,
        Length::Fill(1),
        Length::Fit,
        Kind::Row {
            spacing,
            cross_align: align,
        },
        children,
    )
}

/// A fill-height column node over caller-owned children.
pub(crate) fn column<'a>(
    id: NodeId,
    children: &'a [Node<'a>],
    spacing: u16,
    align: Align,
) -> Node<'a> {
    Node::container(
        id,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Column {
            spacing,
            cross_align: align,
        },
        children,
    )
}

/// A stack node over caller-owned children.
pub(crate) fn stack<'a>(id: NodeId, children: &'a [Node<'a>], align: Align) -> Node<'a> {
    Node::container(
        id,
        Length::Fill(1),
        Length::Fill(1),
        Kind::Stack { align },
        children,
    )
}

/// A leaf node that paints nothing.
pub(crate) fn leaf<'a>(id: NodeId, width: Length, height: Length) -> Node<'a> {
    Node::container(id, width, height, Kind::Leaf, &[])
}

/// A fit-size column node over caller-owned children.
pub(crate) fn column_fit<'a>(
    id: NodeId,
    children: &'a [Node<'a>],
    spacing: u16,
    align: Align,
) -> Node<'a> {
    Node::container(
        id,
        Length::Fit,
        Length::Fit,
        Kind::Column {
            spacing,
            cross_align: align,
        },
        children,
    )
}

/// A uniform grid node over caller-owned children.
pub(crate) fn grid<'a>(
    id: NodeId,
    children: &'a [Node<'a>],
    columns: u8,
    column_spacing: u16,
    row_spacing: u16,
) -> Node<'a> {
    Node::container(
        id,
        Length::Fill(1),
        Length::Fit,
        Kind::Grid {
            columns,
            column_spacing,
            row_spacing,
        },
        children,
    )
}

/// Maximum entries after merging a base screen and an overlay.
pub(crate) const MERGED_CAPACITY: usize = ENTRY_CAPACITY * 2;

/// An empty merged entry array.
pub(crate) const fn merged_entries() -> [LayoutEntry; MERGED_CAPACITY] {
    [EMPTY_ENTRY; MERGED_CAPACITY]
}

/// Copies a base layout and an overlay layout into one paint-ordered array.
pub(crate) fn merge(
    base: &[LayoutEntry],
    overlay: &[LayoutEntry],
    out: &mut [LayoutEntry],
) -> usize {
    let first = base.len().min(out.len());
    out[..first].copy_from_slice(&base[..first]);
    let second = overlay.len().min(out.len().saturating_sub(first));
    out[first..first + second].copy_from_slice(&overlay[..second]);
    first + second
}
