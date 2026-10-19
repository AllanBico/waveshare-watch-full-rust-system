//! Log categories and fixed-capacity page storage.

use common::{Event, LogEntry, Timestamp, WakeReason};

/// Entries retained per category.
pub const LOG_PAGE_SIZE: usize = 64;
/// Entries shown on one watch page.
pub const LOGS_PER_PAGE: usize = 5;
/// Number of visible log categories.
pub const CATEGORY_COUNT: usize = 5;

/// Placeholder used to initialize fixed arrays.
pub(super) const EMPTY_LOG: LogEntry = LogEntry::new(Timestamp::from_millis(0), Event::Error(0));

/// One filterable log category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogCategory {
    /// Wake events other than alarms.
    Wakes,
    /// Alarm wake events.
    Alarms,
    /// Battery voltage samples.
    Battery,
    /// Watch/desk mode switches.
    Modes,
    /// Stable error codes.
    Errors,
}

impl LogCategory {
    /// Every category, in display order.
    pub const ALL: [Self; CATEGORY_COUNT] = [
        Self::Wakes,
        Self::Alarms,
        Self::Battery,
        Self::Modes,
        Self::Errors,
    ];

    /// Dense index used by category pages.
    pub const fn index(self) -> usize {
        match self {
            Self::Wakes => 0,
            Self::Alarms => 1,
            Self::Battery => 2,
            Self::Modes => 3,
            Self::Errors => 4,
        }
    }

    /// Short display label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Wakes => "Wakes",
            Self::Alarms => "Alarms",
            Self::Battery => "Battery",
            Self::Modes => "Mode switches",
            Self::Errors => "Errors",
        }
    }

    /// Whether this category includes the event.
    pub fn matches(self, event: Event) -> bool {
        match self {
            Self::Wakes => matches!(event, Event::Wake(reason) if reason != WakeReason::Alarm),
            Self::Alarms => matches!(event, Event::Wake(WakeReason::Alarm)),
            Self::Battery => matches!(event, Event::Battery(_)),
            Self::Modes => matches!(event, Event::ModeSwitch { .. }),
            Self::Errors => matches!(event, Event::Error(_)),
        }
    }
}

/// One category's newest-first entries.
pub(super) struct LogPage {
    entries: [LogEntry; LOG_PAGE_SIZE],
    len: usize,
}

impl LogPage {
    /// Creates an empty page.
    pub(super) const fn new() -> Self {
        Self {
            entries: [EMPTY_LOG; LOG_PAGE_SIZE],
            len: 0,
        }
    }

    /// Removes every entry.
    pub(super) fn clear(&mut self) {
        self.len = 0;
    }

    /// The stored entries, newest first after `sort_newest_first`.
    pub(super) fn entries(&self) -> &[LogEntry] {
        &self.entries[..self.len]
    }

    /// Whether the page is full.
    pub(super) fn is_full(&self) -> bool {
        self.len >= LOG_PAGE_SIZE
    }

    /// Appends an entry unless it is already present.
    pub(super) fn push_unique(&mut self, entry: LogEntry) {
        if self.len >= LOG_PAGE_SIZE || self.entries().contains(&entry) {
            return;
        }
        self.entries[self.len] = entry;
        self.len += 1;
    }

    /// Sorts entries newest first.
    pub(super) fn sort_newest_first(&mut self) {
        for index in 1..self.len {
            let value = self.entries[index];
            let mut cursor = index;
            while cursor > 0 && self.entries[cursor - 1].timestamp < value.timestamp {
                self.entries[cursor] = self.entries[cursor - 1];
                cursor -= 1;
            }
            self.entries[cursor] = value;
        }
    }
}
