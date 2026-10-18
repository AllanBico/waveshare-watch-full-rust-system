//! Interaction states shared by every widget.

/// A set of widget interaction states.
///
/// States combine: a selected widget can also be focused or pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WidgetState(u8);

impl WidgetState {
    /// No flags; the default resting state.
    pub const NORMAL: Self = Self(0);
    /// Keyboard or rotary focus.
    pub const FOCUSED: Self = Self(1 << 0);
    /// Pointer or touch press.
    pub const PRESSED: Self = Self(1 << 1);
    /// Not interactive.
    pub const DISABLED: Self = Self(1 << 2);
    /// Chosen in a group.
    pub const SELECTED: Self = Self(1 << 3);
    /// Validation or operation failure.
    pub const ERROR: Self = Self(1 << 4);

    /// Adds another state flag.
    pub const fn with(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Removes another state flag.
    pub const fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// Whether every flag in `other` is present.
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether no flags are set.
    pub const fn is_normal(self) -> bool {
        self.0 == 0
    }

    /// Whether the focused flag is set.
    pub const fn is_focused(self) -> bool {
        self.contains(Self::FOCUSED)
    }

    /// Whether the pressed flag is set.
    pub const fn is_pressed(self) -> bool {
        self.contains(Self::PRESSED)
    }

    /// Whether the disabled flag is set.
    pub const fn is_disabled(self) -> bool {
        self.contains(Self::DISABLED)
    }

    /// Whether the selected flag is set.
    pub const fn is_selected(self) -> bool {
        self.contains(Self::SELECTED)
    }

    /// Whether the error flag is set.
    pub const fn is_error(self) -> bool {
        self.contains(Self::ERROR)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_combine_and_clear_independently() {
        let state = WidgetState::NORMAL
            .with(WidgetState::FOCUSED)
            .with(WidgetState::SELECTED);
        assert!(state.is_focused());
        assert!(state.is_selected());
        assert!(!state.is_pressed());
        assert!(state.without(WidgetState::FOCUSED).is_selected());
        assert!(state
            .without(WidgetState::FOCUSED)
            .contains(WidgetState::SELECTED));
    }

    #[test]
    fn normal_has_no_flags() {
        assert!(WidgetState::default().is_normal());
    }
}
