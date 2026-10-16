//! Runtime modes.

/// The watch's runtime mode.
///
/// Chosen at runtime from VBUS detection, never a build variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Mode {
    /// Ultra-low-power mode: monochrome UI, radios off, deep sleep default.
    Watch,
    /// USB-powered mode: color UI, voice, and sync active.
    Desk,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_are_distinct() {
        assert_ne!(Mode::Watch, Mode::Desk);
    }

    #[test]
    fn mode_is_copy() {
        let mode = Mode::Watch;
        let copied = mode;
        assert_eq!(mode, copied);
    }
}
