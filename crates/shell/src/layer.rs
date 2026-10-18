//! Paint-ordered overlay layers and input blocking.

/// Number of layer kinds, from app to toast.
pub const LAYER_CAPACITY: usize = 5;

/// The five layers, from app to toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum LayerKind {
    /// The active app route.
    App = 0,
    /// The app drawer overlay.
    AppDrawer = 1,
    /// The quick settings overlay.
    QuickSettings = 2,
    /// The modal dialog layer.
    Dialog = 3,
    /// The transient toast layer.
    Toast = 4,
}

impl LayerKind {
    /// Number of layer kinds.
    pub const COUNT: usize = LAYER_CAPACITY;

    /// The paint priority, lowest first.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Whether this layer is transient and never takes focus.
    pub const fn is_toast(self) -> bool {
        matches!(self, Self::Toast)
    }

    /// Whether this layer blocks the app underneath.
    pub const fn blocks_app_input(self) -> bool {
        matches!(self, Self::AppDrawer | Self::QuickSettings | Self::Dialog)
    }

    /// Whether this layer is a modal dialog.
    pub const fn is_modal(self) -> bool {
        matches!(self, Self::Dialog)
    }
}

/// One open layer with its content identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Layer {
    /// The layer role.
    pub kind: LayerKind,
    /// Content identifier, interpreted by the layer's UI.
    pub id: u16,
}

impl Layer {
    /// Creates a layer.
    pub const fn new(kind: LayerKind, id: u16) -> Self {
        Self { kind, id }
    }
}

/// A fixed slot per layer kind, in paint order.
///
/// One app layer is implicit: the active route is the app layer. The stack
/// stores overlays and returns them from lowest to highest priority.
pub struct LayerStack {
    slots: [Option<Layer>; LAYER_CAPACITY],
    len: usize,
}

impl LayerStack {
    /// Creates an empty overlay stack.
    pub const fn new() -> Self {
        Self {
            slots: [None; LAYER_CAPACITY],
            len: 0,
        }
    }

    /// Opens or replaces a layer, returning the previous one.
    pub fn open(&mut self, layer: Layer) -> Option<Layer> {
        let index = layer.kind.index();
        let previous = self.slots.get_mut(index)?.replace(layer);
        if previous.is_none() {
            self.len += 1;
        }
        previous
    }

    /// Closes a layer, returning it when it was open.
    pub fn close(&mut self, kind: LayerKind) -> Option<Layer> {
        let previous = self.slots.get_mut(kind.index())?.take();
        if previous.is_some() {
            self.len -= 1;
        }
        previous
    }

    /// Whether a layer kind is open.
    pub fn is_open(&self, kind: LayerKind) -> bool {
        self.slots.get(kind.index()).is_some_and(Option::is_some)
    }

    /// Returns an open layer by kind.
    pub fn get(&self, kind: LayerKind) -> Option<Layer> {
        self.slots.get(kind.index()).copied().flatten()
    }

    /// The number of open overlays.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether no overlay is open.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The topmost layer, including toast.
    pub fn top(&self) -> Option<Layer> {
        self.slots.iter().rev().flatten().copied().next()
    }

    /// The topmost layer that can receive input.
    pub fn top_interactive(&self) -> Option<Layer> {
        self.slots
            .iter()
            .rev()
            .flatten()
            .copied()
            .find(|layer| !layer.kind.is_toast() && layer.kind != LayerKind::App)
    }

    /// Whether any layer blocks input to the app.
    pub fn blocks_app_input(&self) -> bool {
        self.slots
            .iter()
            .flatten()
            .any(|layer| layer.kind.blocks_app_input())
    }

    /// Whether a modal dialog is open.
    pub fn is_modal(&self) -> bool {
        self.is_open(LayerKind::Dialog)
    }

    /// Closes overlays and dialogs, keeping toasts.
    pub fn clear_overlays(&mut self) {
        self.close(LayerKind::AppDrawer);
        self.close(LayerKind::QuickSettings);
        self.close(LayerKind::Dialog);
    }

    /// Closes every overlay, dialog, and toast.
    pub fn clear_all(&mut self) {
        *self = Self::new();
    }
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}
