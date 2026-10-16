//! Wall-clock time.

/// Milliseconds in one second.
const MILLIS_PER_SEC: u64 = 1_000;

/// A point in time, in milliseconds since the Unix epoch (UTC).
///
/// Source-agnostic: the RTC fills it once available. Monotonic uptime is a
/// separate concern, tracked with `embassy_time::Instant`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Timestamp(u64);

impl Timestamp {
    /// Creates a timestamp from milliseconds since the Unix epoch.
    pub const fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    /// Creates a timestamp from seconds since the Unix epoch, saturating.
    pub const fn from_secs(secs: u64) -> Self {
        Self(secs.saturating_mul(MILLIS_PER_SEC))
    }

    /// Milliseconds since the Unix epoch.
    pub const fn as_millis(self) -> u64 {
        self.0
    }

    /// Whole seconds since the Unix epoch.
    pub const fn as_secs(self) -> u64 {
        self.0 / MILLIS_PER_SEC
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn millis_roundtrip() {
        assert_eq!(Timestamp::from_millis(1_234).as_millis(), 1_234);
    }

    #[test]
    fn secs_convert_to_millis() {
        assert_eq!(Timestamp::from_secs(2).as_millis(), 2_000);
    }

    #[test]
    fn secs_truncate_sub_second() {
        assert_eq!(Timestamp::from_millis(1_999).as_secs(), 1);
    }

    #[test]
    fn zero_is_valid() {
        assert_eq!(Timestamp::from_millis(0).as_secs(), 0);
    }

    #[test]
    fn max_roundtrips() {
        assert_eq!(Timestamp::from_millis(u64::MAX).as_millis(), u64::MAX);
    }

    #[test]
    fn from_secs_saturates_instead_of_overflowing() {
        assert_eq!(Timestamp::from_secs(u64::MAX).as_millis(), u64::MAX);
    }
}
