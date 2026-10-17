//! Host tests for the time-series layer.

use common::{Event, LogEntry, Mode, Timestamp, WakeReason};

use super::*;
use crate::test_support::{MockFlash, MockHandle};
use crate::time_series::{sequence_key, LATEST_KEY, OLDEST_KEY};

const LOG_FLASH_SIZE: usize = 16 * REGION_SIZE;

fn leaked_buffer() -> &'static mut [u8; REGION_SIZE] {
    Box::leak(Box::new([0u8; REGION_SIZE]))
}

fn open_series(size: usize) -> (TimeSeries<'static, MockFlash>, MockHandle) {
    let flash = MockFlash::new(size);
    let handle = flash.handle();
    let (series, outcome) =
        TimeSeries::open(flash, leaked_buffer(), Provisioning::Allowed).unwrap();
    assert_eq!(outcome, OpenOutcome::Provisioned);
    (series, handle)
}

fn entry(millis: u64, event: Event) -> LogEntry {
    LogEntry::new(Timestamp::from_millis(millis), event)
}

fn wake(millis: u64) -> LogEntry {
    entry(millis, Event::Wake(WakeReason::Timer))
}

#[test]
fn append_100_entries_in_order() {
    let (mut series, _) = open_series(LOG_FLASH_SIZE);
    for millis in 0..100u64 {
        let event = if millis % 10 == 0 {
            Event::ModeSwitch {
                from: Mode::Watch,
                to: Mode::Desk,
            }
        } else {
            Event::Wake(WakeReason::Timer)
        };
        let sequence = series.append_log(entry(millis, event)).unwrap();
        assert_eq!(sequence, millis + 1);
    }

    assert_eq!(series.latest().unwrap(), 100);
    assert_eq!(series.oldest().unwrap(), 1);

    let mut entries = Vec::new();
    let last = series
        .read_logs_since(0, |record| {
            entries.push(record);
            true
        })
        .unwrap();
    assert_eq!(last, 100);
    assert_eq!(entries.len(), 100);
    for (index, record) in entries.iter().enumerate() {
        assert_eq!(record.timestamp.as_millis(), index as u64);
        if index % 10 == 0 {
            assert!(matches!(record.event, Event::ModeSwitch { .. }));
        } else {
            assert_eq!(record.event, Event::Wake(WakeReason::Timer));
        }
    }
}

#[test]
fn checkpoint_only_returns_new_entries() {
    let (mut series, _) = open_series(LOG_FLASH_SIZE);
    for millis in 0..10u64 {
        series.append_log(wake(millis)).unwrap();
    }

    let mut entries = Vec::new();
    let last = series
        .read_logs_since(5, |record| {
            entries.push(record);
            true
        })
        .unwrap();
    assert_eq!(last, 10);
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].timestamp.as_millis(), 5);
}

#[test]
fn prune_advances_oldest() {
    let (mut series, _) = open_series(LOG_FLASH_SIZE);
    for millis in 0..10u64 {
        series.append_log(wake(millis)).unwrap();
    }

    assert_eq!(series.prune_logs(Timestamp::from_millis(5)).unwrap(), 5);
    assert_eq!(series.oldest().unwrap(), 6);
    assert_eq!(series.latest().unwrap(), 10);

    let mut entries = Vec::new();
    let last = series
        .read_logs_since(0, |record| {
            entries.push(record);
            true
        })
        .unwrap();
    assert_eq!(last, 10);
    assert_eq!(entries.len(), 5);
    assert_eq!(entries[0].timestamp.as_millis(), 5);
}

#[test]
fn prune_all_leaves_an_empty_log() {
    let (mut series, _) = open_series(LOG_FLASH_SIZE);
    for millis in 0..10u64 {
        series.append_log(wake(millis)).unwrap();
    }

    assert_eq!(series.prune_logs(Timestamp::from_millis(100)).unwrap(), 10);
    assert_eq!(series.oldest().unwrap(), 11);
    assert_eq!(series.latest().unwrap(), 10);
    assert!(series.is_empty().unwrap());

    let mut called = false;
    let last = series
        .read_logs_since(0, |_| {
            called = true;
            true
        })
        .unwrap();
    assert_eq!(last, 0);
    assert!(!called);
}

#[test]
fn prune_nothing_is_a_noop() {
    let (mut series, _) = open_series(LOG_FLASH_SIZE);
    for millis in 0..3u64 {
        series.append_log(wake(millis)).unwrap();
    }
    assert_eq!(series.prune_logs(Timestamp::from_millis(0)).unwrap(), 0);
    assert_eq!(series.oldest().unwrap(), 1);
}

#[test]
fn callback_can_stop_early() {
    let (mut series, _) = open_series(LOG_FLASH_SIZE);
    for millis in 0..3u64 {
        series.append_log(wake(millis)).unwrap();
    }

    let mut seen = 0;
    let last = series
        .read_logs_since(0, |_| {
            seen += 1;
            seen < 3
        })
        .unwrap();
    assert_eq!(seen, 3);
    assert_eq!(last, 2);
}

#[test]
fn index_survives_reopen() {
    let (mut series, handle) = open_series(LOG_FLASH_SIZE);
    for millis in 0..3u64 {
        series.append_log(wake(millis)).unwrap();
    }
    drop(series);

    let flash = MockFlash::from_handle(handle);
    let (mut series, outcome) =
        TimeSeries::open(flash, leaked_buffer(), Provisioning::Forbidden).unwrap();
    assert_eq!(outcome, OpenOutcome::Ready);
    assert_eq!(series.latest().unwrap(), 3);
    assert_eq!(series.oldest().unwrap(), 1);

    let mut entries = Vec::new();
    series
        .read_logs_since(1, |record| {
            entries.push(record);
            true
        })
        .unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn read_skips_missing_sequences() {
    let flash = MockFlash::new(LOG_FLASH_SIZE);
    let (mut store, _) = TicKvStorage::open(flash, leaked_buffer(), Provisioning::Allowed).unwrap();

    let mut value = [0u8; common::MAX_ENCODED_LOG_LEN];
    let first = wake(1);
    let length = common::encode_log(&first, &mut value).unwrap();
    store.set(&sequence_key(1), &value[..length]).unwrap();

    let third = wake(3);
    let length = common::encode_log(&third, &mut value).unwrap();
    store.set(&sequence_key(3), &value[..length]).unwrap();
    store.set(LATEST_KEY, &3u64.to_le_bytes()).unwrap();
    store.set(OLDEST_KEY, &1u64.to_le_bytes()).unwrap();

    let mut series = TimeSeries::from_store(store);
    let mut entries = Vec::new();
    let last = series
        .read_logs_since(0, |record| {
            entries.push(record);
            true
        })
        .unwrap();
    assert_eq!(last, 3);
    assert_eq!(entries.len(), 2);
}

#[test]
fn corrupt_records_are_rejected() {
    let flash = MockFlash::new(LOG_FLASH_SIZE);
    let (mut store, _) = TicKvStorage::open(flash, leaked_buffer(), Provisioning::Allowed).unwrap();
    let mut corrupt = [0u8; 10];
    corrupt[0] = common::SCHEMA_VERSION;
    corrupt[9] = 0xFF;
    store.set(&sequence_key(1), &corrupt).unwrap();
    store.set(LATEST_KEY, &1u64.to_le_bytes()).unwrap();
    store.set(OLDEST_KEY, &1u64.to_le_bytes()).unwrap();

    let mut series = TimeSeries::from_store(store);
    assert!(matches!(
        series.read_logs_since(0, |_| true),
        Err(Error::Corrupt)
    ));
}

#[test]
fn entries_with_a_wrong_schema_are_rejected() {
    let (mut series, _) = open_series(LOG_FLASH_SIZE);
    let mut record = wake(1);
    record.schema_version = common::SCHEMA_VERSION + 1;
    assert!(matches!(
        series.append_log(record),
        Err(Error::VersionMismatch)
    ));
}
