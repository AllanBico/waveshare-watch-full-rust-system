//! Fixed-capacity sorted alarm set.

use common::Timestamp;

/// Maximum alarms retained by the watch.
pub const MAX_ALARMS: usize = 4;

/// Immediate re-wake when an alarm became due while the device was awake.
const DUE_WAKE_MS: u32 = 10;

/// A set of absolute one-shot alarm times, earliest first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlarmSet {
    alarms: [Option<Timestamp>; MAX_ALARMS],
}

impl AlarmSet {
    /// Creates an empty set.
    pub const fn new() -> Self {
        Self {
            alarms: [None; MAX_ALARMS],
        }
    }

    /// Number of alarms.
    pub fn len(&self) -> usize {
        self.alarms.iter().flatten().count()
    }

    /// Whether the set is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The alarms in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = Timestamp> + '_ {
        self.alarms.iter().flatten().copied()
    }

    /// Adds an alarm, ignoring duplicates.
    pub fn add(&mut self, at: Timestamp) -> bool {
        if self.contains(at) {
            return true;
        }
        let mut insert = self.len();
        for index in 0..self.len() {
            if self.alarms[index].is_some_and(|existing| at < existing) {
                insert = index;
                break;
            }
        }
        if insert == MAX_ALARMS {
            return false;
        }
        let len = self.len();
        for index in (insert..len).rev() {
            self.alarms[index + 1] = self.alarms[index];
        }
        self.alarms[insert] = Some(at);
        true
    }

    /// Removes an alarm, returning whether it existed.
    pub fn remove(&mut self, at: Timestamp) -> bool {
        let Some(index) = self.position(at) else {
            return false;
        };
        for index in index..MAX_ALARMS - 1 {
            self.alarms[index] = self.alarms[index + 1];
        }
        self.alarms[MAX_ALARMS - 1] = None;
        true
    }

    /// The earliest alarm at or before `now`.
    pub fn due(&self, now: Timestamp) -> Option<Timestamp> {
        self.iter().find(|at| *at <= now)
    }

    /// The earliest alarm after `now`.
    pub fn next_after(&self, now: Timestamp) -> Option<Timestamp> {
        self.iter().find(|at| *at > now)
    }

    /// The RTC sleep interval before the next alarm, if one is set.
    pub fn next_wake_in_ms(&self, now: Timestamp) -> Option<u32> {
        if self.due(now).is_some() {
            return Some(DUE_WAKE_MS);
        }
        let next = self.next_after(now)?;
        let delta = next.as_millis().saturating_sub(now.as_millis());
        Some(u32::try_from(delta).unwrap_or(u32::MAX))
    }

    fn contains(&self, at: Timestamp) -> bool {
        self.position(at).is_some()
    }

    fn position(&self, at: Timestamp) -> Option<usize> {
        self.alarms[..self.len()]
            .iter()
            .position(|entry| *entry == Some(at))
    }
}

impl Default for AlarmSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: u64) -> Timestamp {
        Timestamp::from_secs(secs)
    }

    #[test]
    fn alarms_stay_sorted_and_ignore_duplicates() {
        let mut set = AlarmSet::new();
        assert!(set.add(at(30)));
        assert!(set.add(at(10)));
        assert!(set.add(at(20)));
        assert!(set.add(at(20)));
        assert_eq!(set.iter().collect::<Vec<_>>(), vec![at(10), at(20), at(30)]);
    }

    #[test]
    fn a_full_set_rejects_one_more() {
        let mut set = AlarmSet::new();
        for index in 0..MAX_ALARMS {
            assert!(set.add(at(index as u64)));
        }
        assert!(!set.add(at(99)));
    }

    #[test]
    fn due_and_next_split_at_now() {
        let mut set = AlarmSet::new();
        set.add(at(10));
        set.add(at(20));
        assert_eq!(set.due(at(10)), Some(at(10)));
        assert_eq!(set.next_after(at(10)), Some(at(20)));
        assert_eq!(set.next_after(at(20)), None);
    }

    #[test]
    fn next_wake_follows_the_alarm_set() {
        let mut set = AlarmSet::new();
        assert_eq!(set.next_wake_in_ms(at(0)), None);
        set.add(at(100));
        assert_eq!(set.next_wake_in_ms(at(0)), Some(100_000));
        assert_eq!(set.next_wake_in_ms(at(90)), Some(10_000));
    }

    #[test]
    fn missed_alarms_are_still_due() {
        let mut set = AlarmSet::new();
        set.add(at(10));
        assert_eq!(set.due(at(1_000)), Some(at(10)));
    }

    #[test]
    fn an_alarm_that_became_due_while_awake_wakes_immediately() {
        let mut set = AlarmSet::new();
        set.add(at(10));
        assert_eq!(set.next_wake_in_ms(at(11)), Some(DUE_WAKE_MS));
    }
}
