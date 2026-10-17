//! Retained byte ring of encoded log entries.
//!
//! The header is a small magic/version plus head, count, and dropped counters.
//! Slots are fixed-size so the ring survives resets without allocation.

use common::{decode_log, encode_log, LogEntry, LogError, MAX_ENCODED_LOG_LEN};

/// Bytes per ring slot: one length byte plus the largest encoded record.
pub const SLOT_BYTES: usize = MAX_ENCODED_LOG_LEN + 1;
/// Number of retained slots.
pub const CAPACITY: usize = 96;
const HEADER_BYTES: usize = 12;
/// Bytes needed for the retained ring.
pub const RETAINED_BYTES: usize = HEADER_BYTES + SLOT_BYTES * CAPACITY;

/// The storage type for a retained ring.
pub type Retained = [u8; RETAINED_BYTES];

const MAGIC: [u8; 4] = *b"WV2L";
const VERSION: u8 = 1;
const MAGIC_OFFSET: usize = 0;
const VERSION_OFFSET: usize = 4;
const HEAD_OFFSET: usize = 6;
const COUNT_OFFSET: usize = 8;
const DROPPED_OFFSET: usize = 10;
const SLOT_DATA: usize = SLOT_BYTES - 1;

/// A bounded ring over retained memory.
pub struct Ring<'a> {
    bytes: &'a mut Retained,
}

impl<'a> Ring<'a> {
    /// Opens the ring, resetting it if the retained memory is not ours.
    pub fn new(bytes: &'a mut Retained) -> Self {
        let mut ring = Self { bytes };
        let valid = ring.bytes[MAGIC_OFFSET..VERSION_OFFSET] == MAGIC
            && ring.bytes[VERSION_OFFSET] == VERSION
            && ring.head() < CAPACITY
            && usize::from(ring.count()) <= CAPACITY;
        if !valid {
            ring.reset();
        }
        ring
    }

    /// Number of valid records.
    pub fn len(&self) -> usize {
        usize::from(self.count())
    }

    /// Whether the ring is empty.
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Records dropped since the last clear.
    pub fn dropped(&self) -> u16 {
        self.dropped_raw()
    }

    /// Appends an entry, dropping the oldest record when full.
    pub fn push(&mut self, entry: &LogEntry) -> Result<(), LogError> {
        let mut payload = [0u8; SLOT_DATA];
        let length = encode_log(entry, &mut payload)?;

        if self.count() == CAPACITY as u16 {
            self.pop();
            self.set_dropped(self.dropped_raw().saturating_add(1));
        }

        let slot = self.head();
        let base = HEADER_BYTES + slot * SLOT_BYTES;
        self.bytes[base] = length as u8;
        self.bytes[base + 1..base + 1 + length].copy_from_slice(&payload[..length]);
        for byte in &mut self.bytes[base + 1 + length..base + SLOT_BYTES] {
            *byte = 0;
        }

        self.set_head((slot + 1) % CAPACITY);
        self.set_count(self.count() + 1);
        Ok(())
    }

    /// Returns the oldest record without removing it.
    pub fn peek(&self) -> Result<Option<LogEntry>, LogError> {
        if self.count() == 0 {
            return Ok(None);
        }
        let slot = (self.head() + CAPACITY - usize::from(self.count())) % CAPACITY;
        let base = HEADER_BYTES + slot * SLOT_BYTES;
        let length = usize::from(self.bytes[base]);
        if length == 0 || length > SLOT_DATA {
            return Err(LogError::Corrupt);
        }
        decode_log(&self.bytes[base + 1..base + 1 + length]).map(Some)
    }

    /// Removes the oldest record, if any.
    pub fn pop(&mut self) {
        if self.count() > 0 {
            self.set_count(self.count() - 1);
        }
    }

    /// Empties the ring and resets the dropped counter.
    pub fn clear(&mut self) {
        self.set_head(0);
        self.set_count(0);
        self.set_dropped(0);
    }

    fn reset(&mut self) {
        self.bytes[MAGIC_OFFSET..VERSION_OFFSET].copy_from_slice(&MAGIC);
        self.bytes[VERSION_OFFSET] = VERSION;
        self.bytes[VERSION_OFFSET + 1] = 0;
        self.clear();
    }

    fn head(&self) -> usize {
        usize::from(read_u16(self.bytes, HEAD_OFFSET))
    }

    fn set_head(&mut self, value: usize) {
        write_u16(self.bytes, HEAD_OFFSET, value as u16);
    }

    fn count(&self) -> u16 {
        read_u16(self.bytes, COUNT_OFFSET)
    }

    fn set_count(&mut self, value: u16) {
        write_u16(self.bytes, COUNT_OFFSET, value);
    }

    fn dropped_raw(&self) -> u16 {
        read_u16(self.bytes, DROPPED_OFFSET)
    }

    fn set_dropped(&mut self, value: u16) {
        write_u16(self.bytes, DROPPED_OFFSET, value);
    }
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    let raw = value.to_le_bytes();
    bytes[offset] = raw[0];
    bytes[offset + 1] = raw[1];
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{Event, Timestamp, WakeReason};

    fn retained() -> Retained {
        [0u8; RETAINED_BYTES]
    }

    fn entry(seconds: u64) -> LogEntry {
        LogEntry::new(
            Timestamp::from_secs(seconds),
            Event::Wake(WakeReason::Timer),
        )
    }

    #[test]
    fn garbage_initializes_to_empty() {
        let mut bytes = [0xAAu8; RETAINED_BYTES];
        let ring = Ring::new(&mut bytes);
        assert!(ring.is_empty());
        assert_eq!(ring.dropped(), 0);
    }

    #[test]
    fn push_peek_pop_round_trip() {
        let mut bytes = retained();
        let mut ring = Ring::new(&mut bytes);
        ring.push(&entry(1)).unwrap();
        ring.push(&entry(2)).unwrap();

        assert_eq!(ring.len(), 2);
        assert_eq!(ring.peek().unwrap().unwrap().timestamp.as_secs(), 1);
        ring.pop();
        assert_eq!(ring.peek().unwrap().unwrap().timestamp.as_secs(), 2);
        ring.pop();
        assert!(ring.peek().unwrap().is_none());
    }

    #[test]
    fn reopen_keeps_records() {
        let mut bytes = retained();
        {
            let mut ring = Ring::new(&mut bytes);
            ring.push(&entry(4)).unwrap();
            ring.push(&entry(5)).unwrap();
        }
        let ring = Ring::new(&mut bytes);
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.peek().unwrap().unwrap().timestamp.as_secs(), 4);
    }

    #[test]
    fn full_ring_drops_the_oldest() {
        let mut bytes = retained();
        let mut ring = Ring::new(&mut bytes);
        for seconds in 0..(CAPACITY as u64 + 5) {
            ring.push(&entry(seconds)).unwrap();
        }
        assert_eq!(ring.len(), CAPACITY);
        assert_eq!(ring.dropped(), 5);
        assert_eq!(ring.peek().unwrap().unwrap().timestamp.as_secs(), 5);
    }

    #[test]
    fn clear_resets_counters() {
        let mut bytes = retained();
        let mut ring = Ring::new(&mut bytes);
        ring.push(&entry(1)).unwrap();
        ring.clear();
        assert!(ring.is_empty());
        assert_eq!(ring.dropped(), 0);
    }

    #[test]
    fn corrupt_slot_is_reported() {
        let mut bytes = retained();
        let mut ring = Ring::new(&mut bytes);
        ring.push(&entry(1)).unwrap();
        ring.bytes[HEADER_BYTES] = SLOT_DATA as u8 + 1;
        assert!(matches!(ring.peek(), Err(LogError::Corrupt)));
    }
}
