//! Boot-time logging: retained ring, battery sample, and time-series flush.

use common::{Event, LogEntry, Millivolts, Mode, Timestamp, WakeReason};
use drivers::flash::{FlashDevice, PartitionFlash};
use esp_println::println;
use static_cell::StaticCell;
use storage::{Provisioning, TimeSeries, REGION_SIZE};

use crate::retained;

const LOG_LABEL: &str = "logs";
/// Flush this many records in watch mode, roughly sixteen wakes.
const WATCH_FLUSH_RECORDS: usize = 32;
/// Below this voltage a watch-mode flash write is not worth the risk.
pub(crate) const STORAGE_WRITE_MIN_MV: u16 = 3_500;

static LOG_BUFFER: StaticCell<[u8; REGION_SIZE]> = StaticCell::new();

/// Result of the boot logging pass.
pub struct LogReport {
    /// Latest persisted sequence, when the logs store is available.
    pub latest: Option<u32>,
    /// Records still waiting in retained memory.
    pub buffered: u16,
    /// Records dropped by ring overflow since the last clear.
    pub dropped: u16,
}

/// Records this boot in the retained ring and flushes according to policy.
///
/// Desk builds flush every boot. Watch builds flush when the ring reaches
/// [`WATCH_FLUSH_RECORDS`], and only above the write voltage.
pub fn run(
    device: &FlashDevice,
    mode: Mode,
    battery_mv: u16,
    wake: Option<WakeReason>,
    now: Timestamp,
) -> LogReport {
    retained::with_ring(|ring| {
        let _ = ring.push(&LogEntry::new(
            now,
            Event::Wake(wake.unwrap_or(WakeReason::Unknown)),
        ));
        let _ = ring.push(&LogEntry::new(
            now,
            Event::Battery(Millivolts::new(battery_mv)),
        ));

        let dropped_before = ring.dropped();
        let region = match device.partition(LOG_LABEL) {
            Ok(region) => region,
            Err(error) => return fail(ring, mode, error.to_common(), "logs partition", now),
        };

        let buffer = LOG_BUFFER.init([0u8; REGION_SIZE]);
        let provisioning = if mode == Mode::Desk {
            Provisioning::Force
        } else {
            Provisioning::Forbidden
        };
        let (mut series, _) = match TimeSeries::open(region, buffer, provisioning) {
            Ok(opened) => opened,
            Err(error) => return fail(ring, mode, error.to_common(), "logs open", now),
        };

        let mut latest = series
            .latest()
            .ok()
            .filter(|sequence| *sequence > 0)
            .and_then(|sequence| u32::try_from(sequence).ok());
        let should_flush = mode == Mode::Desk || ring.len() >= WATCH_FLUSH_RECORDS;
        let can_write = mode == Mode::Desk || battery_mv >= STORAGE_WRITE_MIN_MV;

        if should_flush && can_write {
            if let Some(sequence) = flush(ring, &mut series) {
                latest = Some(sequence);
            }
        }

        let dropped = dropped_before.max(ring.dropped());
        if ring.is_empty() {
            ring.clear();
        }
        LogReport {
            latest,
            buffered: ring.len() as u16,
            dropped,
        }
    })
}

fn flush(
    ring: &mut logging::Ring<'_>,
    series: &mut TimeSeries<'_, PartitionFlash<'_>>,
) -> Option<u32> {
    let mut last = None;
    loop {
        match ring.peek() {
            Ok(Some(entry)) => match series.append_log(entry) {
                Ok(sequence) => {
                    last = u32::try_from(sequence).ok();
                    ring.pop();
                }
                Err(_) => break,
            },
            Ok(None) => break,
            Err(_) => ring.pop(),
        }
    }
    last
}

fn fail(
    ring: &mut logging::Ring<'_>,
    mode: Mode,
    error: common::Error,
    what: &str,
    now: Timestamp,
) -> LogReport {
    report(mode, error, what);
    if error != common::Error::NotReady {
        let _ = ring.push(&LogEntry::new(now, Event::Error(error.code())));
    }
    snapshot(ring, None)
}

fn snapshot(ring: &logging::Ring<'_>, latest: Option<u32>) -> LogReport {
    LogReport {
        latest,
        buffered: ring.len() as u16,
        dropped: ring.dropped(),
    }
}

fn report(mode: Mode, error: common::Error, what: &str) {
    if mode == Mode::Desk {
        println!("[log] {} failed: {}", what, error);
    }
}
