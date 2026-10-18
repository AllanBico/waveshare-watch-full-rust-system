//! Common monochrome widgets and screen patterns.
//!
//! Widgets are pure data: an id, an enum-backed kind with immutable props, and
//! a set of interaction flags. They never allocate, never touch hardware, and
//! never own application state. Build layout first, then paint through the
//! shared luminance painter.

mod arena;
mod button;
mod checkbox;
mod hit;
mod input;
mod kind;
mod list_item;
mod measure;
mod overlay;
mod paint;
mod patterns;
mod progress;
mod radio;
mod slider;
mod state;
mod status;
mod tabs;
mod toggle;

#[cfg(test)]
mod tests;

use core::hash::{Hash, Hasher};

pub use arena::{WidgetArena, WidgetError, WIDGET_CAPACITY};
pub use button::{ButtonProps, IconButtonProps};
pub use checkbox::CheckboxProps;
pub use hit::{collect_hits, HitRegistry, HitTarget, MAX_HIT_TARGETS};
pub use input::InputProps;
pub use kind::WidgetKind;
pub use list_item::ListItemProps;
pub use overlay::{ContextMenuProps, DialogProps, MenuItem, ToastProps};
pub use paint::{paint_widget, paint_widgets};
pub use patterns::{
    ConfirmProps, EmptyStateProps, ErrorStateProps, LoadingStateProps, ScreenScaffoldProps,
};
pub use progress::{BadgeProps, ProgressProps};
pub use radio::RadioProps;
pub use slider::SliderProps;
pub use state::WidgetState;
pub use status::{Status, StatusIndicatorProps};
pub use tabs::TabsProps;
pub use toggle::ToggleProps;

use crate::assets::IconId;
use crate::scene::WidgetId;

/// One widget in paint order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Widget<'a> {
    /// Stable widget identifier.
    pub id: WidgetId,
    /// Immutable props and component kind.
    pub kind: WidgetKind<'a>,
    /// Interaction state flags.
    pub state: WidgetState,
}

impl<'a> Widget<'a> {
    /// Creates a widget in the normal state.
    pub const fn new(id: WidgetId, kind: WidgetKind<'a>) -> Self {
        Self {
            id,
            kind,
            state: WidgetState::NORMAL,
        }
    }

    /// Replaces the interaction state flags.
    pub const fn with_state(mut self, state: WidgetState) -> Self {
        self.state = state;
        self
    }

    /// A stable hash of props and state for scene diffing.
    pub fn fingerprint(&self) -> u64 {
        let mut hasher = Fnv::new();
        self.kind.hash(&mut hasher);
        self.state.hash(&mut hasher);
        hasher.finish()
    }
}

/// A small FNV-1a hasher for deterministic fingerprints.
struct Fnv(u64);

impl Fnv {
    const fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hash for IconId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*self as u8).hash(state);
    }
}

impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}
