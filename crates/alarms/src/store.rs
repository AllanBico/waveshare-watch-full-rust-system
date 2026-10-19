//! Alarm persistence over a key-value backend.

use common::Timestamp;
use hal::Storage;

use crate::{AlarmSet, MAX_ALARMS};

/// Key holding the encoded alarm set.
pub const ALARM_KEY: &[u8] = b"alarms/v1";
/// Schema version for the encoded set.
const VERSION: u8 = 1;
/// Version plus count plus every timestamp.
const MAX_ENCODED_LEN: usize = 2 + MAX_ALARMS * 8;

/// A store write failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    /// The backend rejected the write or is unavailable.
    Storage,
}

/// What happened while loading alarms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LoadReport {
    /// Whether a stored set was found.
    pub loaded: bool,
    /// Whether the stored set was corrupt or unreadable.
    pub failed: bool,
}

/// An alarm set mirrored to persistent storage.
pub struct AlarmStore<B> {
    backend: Option<B>,
    set: AlarmSet,
    dirty: bool,
}

impl<B: Storage> AlarmStore<B> {
    /// Loads the stored alarms, falling back to an empty set.
    pub fn load(mut backend: B) -> (Self, LoadReport) {
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let mut report = LoadReport::default();
        let set = match backend.get(ALARM_KEY, &mut buffer) {
            Ok(length) => match decode(&buffer[..length]) {
                Ok(set) => {
                    report.loaded = true;
                    set
                }
                Err(()) => {
                    report.failed = true;
                    AlarmSet::new()
                }
            },
            Err(_) => AlarmSet::new(),
        };
        (
            Self {
                backend: Some(backend),
                set,
                dirty: false,
            },
            report,
        )
    }

    /// Creates a store with no writable backend.
    pub const fn unavailable() -> Self {
        Self {
            backend: None,
            set: AlarmSet::new(),
            dirty: false,
        }
    }

    /// The in-memory alarm set.
    pub const fn set(&self) -> &AlarmSet {
        &self.set
    }

    /// Adds an alarm and marks the store dirty.
    pub fn add(&mut self, at: Timestamp) -> bool {
        let added = self.set.add(at);
        self.dirty |= added;
        added
    }

    /// Removes an alarm and marks the store dirty when it existed.
    pub fn remove(&mut self, at: Timestamp) -> bool {
        let removed = self.set.remove(at);
        self.dirty |= removed;
        removed
    }

    /// The earliest alarm at or before `now`.
    pub fn due(&self, now: Timestamp) -> Option<Timestamp> {
        self.set.due(now)
    }

    /// The RTC sleep interval before the next alarm or housekeeping timer.
    pub fn next_wake_in_ms(&self, now: Timestamp, housekeeping_ms: u32) -> u32 {
        self.set.next_wake_in_ms(now, housekeeping_ms)
    }

    /// Writes pending changes, returning how many values were stored.
    pub fn flush(&mut self) -> Result<usize, StoreError> {
        if !self.dirty {
            return Ok(0);
        }
        let Some(backend) = self.backend.as_mut() else {
            return Err(StoreError::Storage);
        };
        let mut buffer = [0u8; MAX_ENCODED_LEN];
        let length = encode(&self.set, &mut buffer);
        backend
            .set(ALARM_KEY, &buffer[..length])
            .map_err(|_| StoreError::Storage)?;
        self.dirty = false;
        Ok(1)
    }
}

fn encode(set: &AlarmSet, out: &mut [u8; MAX_ENCODED_LEN]) -> usize {
    out[0] = VERSION;
    out[1] = set.len() as u8;
    let mut length = 2;
    for at in set.iter() {
        out[length..length + 8].copy_from_slice(&at.as_millis().to_le_bytes());
        length += 8;
    }
    length
}

fn decode(bytes: &[u8]) -> Result<AlarmSet, ()> {
    if bytes.len() < 2 || bytes[0] != VERSION {
        return Err(());
    }
    let count = usize::from(bytes[1]);
    if count > MAX_ALARMS || bytes.len() != 2 + count * 8 {
        return Err(());
    }
    let mut set = AlarmSet::new();
    for index in 0..count {
        let start = 2 + index * 8;
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&bytes[start..start + 8]);
        if !set.add(Timestamp::from_millis(u64::from_le_bytes(raw))) {
            return Err(());
        }
    }
    Ok(set)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Clone, Default)]
    struct FakeStorage {
        value: Rc<RefCell<Vec<u8>>>,
    }

    impl Storage for FakeStorage {
        type Error = ();

        fn get(&mut self, _key: &[u8], value: &mut [u8]) -> Result<usize, Self::Error> {
            let stored = self.value.borrow();
            if stored.is_empty() || value.len() < stored.len() {
                return Err(());
            }
            value[..stored.len()].copy_from_slice(&stored);
            Ok(stored.len())
        }

        fn set(&mut self, _key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
            *self.value.borrow_mut() = value.to_vec();
            Ok(())
        }

        fn delete(&mut self, _key: &[u8]) -> Result<bool, Self::Error> {
            let existed = !self.value.borrow().is_empty();
            self.value.borrow_mut().clear();
            Ok(existed)
        }
    }

    fn at(secs: u64) -> Timestamp {
        Timestamp::from_secs(secs)
    }

    #[test]
    fn alarms_survive_a_reload() {
        let storage = FakeStorage::default();
        let (mut store, report) = AlarmStore::load(storage.clone());
        assert!(!report.loaded);
        store.add(at(60));
        assert_eq!(store.flush().unwrap(), 1);

        let (store, report) = AlarmStore::load(storage);
        assert!(report.loaded);
        assert_eq!(store.due(at(60)), Some(at(60)));
    }

    #[test]
    fn removing_due_alarms_persists_the_disable() {
        let storage = FakeStorage::default();
        let (mut store, _) = AlarmStore::load(storage.clone());
        store.add(at(60));
        store.flush().unwrap();
        assert!(store.remove(at(60)));
        store.flush().unwrap();

        let (store, _) = AlarmStore::load(storage);
        assert!(store.set().is_empty());
    }

    #[test]
    fn a_corrupt_value_loads_empty() {
        let storage = FakeStorage::default();
        *storage.value.borrow_mut() = vec![0xAA, 0xBB];
        let (store, report) = AlarmStore::load(storage);
        assert!(report.failed);
        assert!(store.set().is_empty());
    }
}
