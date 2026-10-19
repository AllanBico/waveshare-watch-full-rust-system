//! Reads the newest logs into the Logs app snapshot.
//!
//! The app never touches flash. This module merges retained pending records
//! with the persistent time-series, deduplicating and sorting in the app.

use apps::{LogCategory, LogsApp};
use drivers::flash::FlashDevice;
use esp_println::println;
use storage::{Provisioning, TimeSeries};

use crate::retained;

const LOG_LABEL: &str = "logs";

/// Loads pending and persisted records into `logs`.
pub fn load(
    category: LogCategory,
    device: Option<&'static FlashDevice>,
    logs: &mut LogsApp,
    log: bool,
) {
    logs.begin_load(category);
    retained::with_ring(|ring| {
        for index in 0..ring.len() {
            if let Ok(Some(entry)) = ring.get_newest(index) {
                logs.ingest(entry);
            }
        }
    });

    let Some(device) = device else {
        logs.finish_load();
        return;
    };
    let region = match device.partition(LOG_LABEL) {
        Ok(region) => region,
        Err(_) => {
            logs.finish_load();
            return;
        }
    };
    let buffer = crate::log_boot::read_buffer();
    let Ok((mut series, _)) = TimeSeries::open(region, buffer, Provisioning::Forbidden) else {
        logs.finish_load();
        return;
    };
    let result = series.read_logs_descending(|entry| {
        logs.ingest(entry);
        !logs.is_full()
    });
    if let Err(error) = result {
        if log {
            println!("[logs] read failed: {}", error);
        }
    }
    logs.finish_load();
}
