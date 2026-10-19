//! Append-only time-series log on top of TicKV.
//!
//! TicKV cannot enumerate keys, so the layer keeps explicit index keys:
//! `log/latest` and `log/oldest`. Entries use fixed-width `log/{sequence}`
//! keys. Reads stream forward from a checkpoint; pruning removes a timestamp
//! prefix. Sequence gaps are skipped, so a partial prune never breaks reads.

use common::{decode_log, encode_log, LogEntry, LogError, Timestamp, MAX_ENCODED_LOG_LEN};
use embedded_storage::nor_flash::NorFlash;
use embedded_storage::ReadStorage;

use super::kv::{OpenOutcome, Provisioning, TicKvStorage};
use super::Error;

/// Key holding the newest sequence.
pub(crate) const LATEST_KEY: &[u8] = b"log/latest";
/// Key holding the oldest sequence still readable.
pub(crate) const OLDEST_KEY: &[u8] = b"log/oldest";

const SEQUENCE_PREFIX: &[u8; 4] = b"log/";
const SEQUENCE_DIGITS: usize = 16;
pub(crate) const SEQUENCE_KEY_LEN: usize = SEQUENCE_PREFIX.len() + SEQUENCE_DIGITS;
const HEX: &[u8; 16] = b"0123456789abcdef";

/// An append-only log over one TicKV store.
pub struct TimeSeries<'a, F: NorFlash + ReadStorage> {
    store: TicKvStorage<'a, F>,
}

impl<'a, F: NorFlash + ReadStorage> TimeSeries<'a, F> {
    /// Opens a log over a flash partition.
    pub fn open(
        region: F,
        buffer: &'a mut [u8; crate::REGION_SIZE],
        provisioning: Provisioning,
    ) -> Result<(Self, OpenOutcome), Error> {
        let (store, outcome) = TicKvStorage::open(region, buffer, provisioning)?;
        Ok((Self { store }, outcome))
    }

    /// Wraps an already opened key-value store.
    pub fn from_store(store: TicKvStorage<'a, F>) -> Self {
        Self { store }
    }

    /// Appends an entry, returning its monotonic sequence number.
    ///
    /// The entry is written before `latest` advances, so a power loss between
    /// the two writes leaves an unreachable orphan that the next append
    /// replaces, not a visible gap.
    pub fn append_log(&mut self, entry: LogEntry) -> Result<u64, Error> {
        let mut value = [0u8; MAX_ENCODED_LOG_LEN];
        let length = encode_log(&entry, &mut value).map_err(map_log)?;

        let latest = self.latest()?;
        let sequence = latest.checked_add(1).ok_or(Error::Full)?;
        let key = sequence_key(sequence);

        self.store.set(&key, &value[..length])?;
        self.store.set(LATEST_KEY, &sequence.to_le_bytes())?;
        if latest == 0 {
            self.store.set(OLDEST_KEY, &1u64.to_le_bytes())?;
        }
        Ok(sequence)
    }

    /// Streams entries after `checkpoint`, returning the last delivered
    /// sequence.
    ///
    /// The callback returns `false` to stop early. Pruned or orphaned gaps are
    /// skipped. If anything after the checkpoint was pruned, reading resumes
    /// at the oldest live entry.
    pub fn read_logs_since<C>(&mut self, checkpoint: u64, mut on_entry: C) -> Result<u64, Error>
    where
        C: FnMut(LogEntry) -> bool,
    {
        let latest = self.latest()?;
        if latest == 0 {
            return Ok(checkpoint);
        }
        let oldest = self.oldest()?;
        let start = checkpoint.max(oldest.saturating_sub(1)).saturating_add(1);
        let mut last = checkpoint;
        let mut sequence = start;

        loop {
            if sequence > latest {
                break;
            }
            let key = sequence_key(sequence);
            let mut value = [0u8; MAX_ENCODED_LOG_LEN];
            match self.store.get(&key, &mut value) {
                Ok(length) => {
                    let entry = decode_log(&value[..length]).map_err(map_log)?;
                    if !on_entry(entry) {
                        break;
                    }
                    last = sequence;
                }
                Err(Error::NotFound) => {}
                Err(Error::BufferTooSmall(_)) => return Err(Error::Corrupt),
                Err(error) => return Err(error),
            }
            if sequence == latest {
                break;
            }
            sequence += 1;
        }
        Ok(last)
    }

    /// Streams entries from newest to oldest.
    ///
    /// The callback returns `false` to stop early. Gaps are skipped. This is
    /// the bounded read path used by the log viewer, which only needs the most
    /// recent records per category.
    pub fn read_logs_descending<C>(&mut self, mut on_entry: C) -> Result<u64, Error>
    where
        C: FnMut(LogEntry) -> bool,
    {
        let latest = self.latest()?;
        if latest == 0 {
            return Ok(0);
        }
        let oldest = self.oldest()?;
        if oldest > latest {
            return Ok(0);
        }

        let mut visited = 0u64;
        let mut sequence = latest;
        loop {
            let key = sequence_key(sequence);
            let mut value = [0u8; MAX_ENCODED_LOG_LEN];
            match self.store.get(&key, &mut value) {
                Ok(length) => {
                    let entry = decode_log(&value[..length]).map_err(map_log)?;
                    visited = visited.saturating_add(1);
                    if !on_entry(entry) {
                        break;
                    }
                }
                Err(Error::NotFound) => {}
                Err(Error::BufferTooSmall(_)) => return Err(Error::Corrupt),
                Err(error) => return Err(error),
            }
            if sequence <= oldest {
                break;
            }
            sequence -= 1;
        }
        Ok(visited)
    }

    /// Removes the oldest live entries until at most `keep` remain.
    ///
    /// Sequence gaps are counted as absent. Returns the number of entries
    /// deleted and advances `oldest` to the first retained sequence.
    pub fn prune_to_count(&mut self, keep: usize) -> Result<usize, Error> {
        let latest = self.latest()?;
        if latest == 0 {
            return Ok(0);
        }
        let oldest = self.oldest()?;
        if oldest > latest {
            return Ok(0);
        }

        let mut live = 0usize;
        let mut sequence = oldest;
        while sequence <= latest {
            let key = sequence_key(sequence);
            let mut value = [0u8; MAX_ENCODED_LOG_LEN];
            match self.store.get(&key, &mut value) {
                Ok(_) => live = live.saturating_add(1),
                Err(Error::NotFound) => {}
                Err(Error::BufferTooSmall(_)) => return Err(Error::Corrupt),
                Err(error) => return Err(error),
            }
            if sequence == latest {
                break;
            }
            sequence += 1;
        }
        if live <= keep {
            return Ok(0);
        }

        let mut remaining = live - keep;
        let mut deleted = 0usize;
        let mut next_oldest = oldest;
        sequence = oldest;
        while sequence <= latest {
            let key = sequence_key(sequence);
            let mut value = [0u8; MAX_ENCODED_LOG_LEN];
            match self.store.get(&key, &mut value) {
                Ok(_) => {
                    if remaining == 0 {
                        break;
                    }
                    self.store.delete(&key)?;
                    deleted = deleted.saturating_add(1);
                    remaining -= 1;
                }
                Err(Error::NotFound) => {}
                Err(Error::BufferTooSmall(_)) => return Err(Error::Corrupt),
                Err(error) => return Err(error),
            }
            next_oldest = sequence.saturating_add(1);
            if sequence == latest {
                break;
            }
            sequence += 1;
        }
        if deleted > 0 {
            self.store.set(OLDEST_KEY, &next_oldest.to_le_bytes())?;
        }
        Ok(deleted)
    }

    /// Invalidates a timestamp prefix and advances `oldest`.
    ///
    /// Pruning stops at the first entry not older than `before`, so entries
    /// must be in timestamp order to prune a full prefix. Space is reclaimed
    /// by a later GC or a `FlashFull` retry.
    pub fn prune_logs(&mut self, before: Timestamp) -> Result<usize, Error> {
        let latest = self.latest()?;
        if latest == 0 {
            return Ok(0);
        }
        let oldest = self.oldest()?;
        if oldest > latest {
            return Ok(0);
        }

        let mut pruned = 0usize;
        let mut keep = oldest;
        let mut sequence = oldest;
        loop {
            let key = sequence_key(sequence);
            let mut value = [0u8; MAX_ENCODED_LOG_LEN];
            match self.store.get(&key, &mut value) {
                Ok(length) => {
                    let entry = decode_log(&value[..length]).map_err(map_log)?;
                    if entry.timestamp < before {
                        self.store.delete(&key)?;
                        pruned += 1;
                        keep = sequence.saturating_add(1);
                    } else {
                        break;
                    }
                }
                Err(Error::NotFound) => keep = sequence.saturating_add(1),
                Err(Error::BufferTooSmall(_)) => return Err(Error::Corrupt),
                Err(error) => return Err(error),
            }
            if sequence == latest {
                break;
            }
            sequence += 1;
        }

        if keep != oldest {
            self.store.set(OLDEST_KEY, &keep.to_le_bytes())?;
        }
        Ok(pruned)
    }

    /// Newest sequence, or `0` when the log has never been written.
    pub fn latest(&mut self) -> Result<u64, Error> {
        self.read_index(LATEST_KEY, 0)
    }

    /// Oldest sequence still readable.
    pub fn oldest(&mut self) -> Result<u64, Error> {
        match self.read_index(OLDEST_KEY, 0)? {
            0 => Ok(1),
            value => Ok(value),
        }
    }

    /// Whether the readable range is empty.
    pub fn is_empty(&mut self) -> Result<bool, Error> {
        Ok(self.oldest()? > self.latest()?)
    }

    /// Reclaims regions with no valid objects.
    pub fn gc(&mut self) -> Result<usize, Error> {
        self.store.gc()
    }

    fn read_index(&mut self, key: &[u8], default: u64) -> Result<u64, Error> {
        let mut raw = [0u8; 8];
        match self.store.get(key, &mut raw) {
            Ok(8) => Ok(u64::from_le_bytes(raw)),
            Ok(_) => Err(Error::Corrupt),
            Err(Error::NotFound) => Ok(default),
            Err(error) => Err(error),
        }
    }
}

const fn map_log(error: LogError) -> Error {
    match error {
        LogError::BufferTooSmall | LogError::Corrupt => Error::Corrupt,
        LogError::VersionMismatch => Error::VersionMismatch,
        _ => Error::Corrupt,
    }
}

/// Builds the fixed-width key for a sequence number.
pub(crate) fn sequence_key(sequence: u64) -> [u8; SEQUENCE_KEY_LEN] {
    let mut key = [0u8; SEQUENCE_KEY_LEN];
    key[..SEQUENCE_PREFIX.len()].copy_from_slice(SEQUENCE_PREFIX);
    for (index, slot) in key[SEQUENCE_PREFIX.len()..].iter_mut().enumerate() {
        let shift = (SEQUENCE_DIGITS - 1 - index) * 4;
        let nibble = ((sequence >> shift) & 0xF) as usize;
        *slot = HEX[nibble];
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequence_keys_are_fixed_width() {
        assert_eq!(&sequence_key(0), b"log/0000000000000000");
        assert_eq!(&sequence_key(0xAB), b"log/00000000000000ab");
        assert_eq!(&sequence_key(u64::MAX), b"log/ffffffffffffffff");
    }
}
