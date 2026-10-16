//! Physical unit newtypes shared across crates.

/// A voltage in millivolts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Millivolts(u16);

impl Millivolts {
    /// Creates a voltage from a raw millivolt value.
    pub const fn new(millivolts: u16) -> Self {
        Self(millivolts)
    }

    /// The raw millivolt value.
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// A battery state of charge, clamped to 0..=100 percent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Percent(u8);

impl Percent {
    /// Creates a percentage, clamping anything above 100.
    pub const fn new(percent: u8) -> Self {
        Self(if percent > 100 { 100 } else { percent })
    }

    /// The clamped percentage value.
    pub const fn value(self) -> u8 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millivolts_roundtrip() {
        assert_eq!(Millivolts::new(3700).value(), 3700);
    }

    #[test]
    fn percent_clamps_above_100() {
        assert_eq!(Percent::new(200).value(), 100);
    }

    #[test]
    fn percent_keeps_valid_value() {
        assert_eq!(Percent::new(42).value(), 42);
    }
}
