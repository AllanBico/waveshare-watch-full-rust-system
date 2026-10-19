//! Logs app grouping, paging, and rendering tests.

use common::{Event, LogEntry, Millivolts, Region, Timestamp, WakeReason};
use ui::{MonoSurface, Theme};

use super::{LogCategory, LogsApp};

fn entry(millis: u64, event: Event) -> LogEntry {
    LogEntry::new(Timestamp::from_millis(millis), event)
}

fn wake(millis: u64) -> LogEntry {
    entry(millis, Event::Wake(WakeReason::Motion))
}

#[test]
fn entries_are_filtered_deduped_and_newest_first() {
    let mut app = LogsApp::new();
    app.begin_load(LogCategory::Wakes);
    app.ingest(wake(1_000));
    app.ingest(wake(3_000));
    app.ingest(wake(3_000));
    app.ingest(entry(2_000, Event::Wake(WakeReason::Alarm)));
    app.ingest(entry(4_000, Event::Battery(Millivolts::new(3_800))));
    app.finish_load();

    assert_eq!(app.count_for_test(), 2);
    assert_eq!(
        app.entry_for_test(0)
            .map(|entry| entry.timestamp.as_millis()),
        Some(3_000)
    );
}

#[test]
fn a_category_keeps_the_latest_sixty_four() {
    let mut app = LogsApp::new();
    app.begin_load(LogCategory::Wakes);
    for millis in (0..100u64).rev() {
        app.ingest(wake(millis));
    }
    app.finish_load();
    assert_eq!(app.count_for_test(), 64);
    assert_eq!(
        app.entry_for_test(0)
            .map(|entry| entry.timestamp.as_millis()),
        Some(99)
    );
    assert_eq!(
        app.entry_for_test(63)
            .map(|entry| entry.timestamp.as_millis()),
        Some(36)
    );
}

#[test]
fn paging_moves_older_and_newer_and_stops_at_the_ends() {
    let mut app = LogsApp::new();
    app.begin_load(LogCategory::Wakes);
    for millis in (0..12u64).rev() {
        app.ingest(wake(millis));
    }
    app.finish_load();

    assert_eq!(app.page_count_for_test(), 3);
    assert!(app.older_for_test());
    assert_eq!(app.page_for_test(), 1);
    assert!(app.older_for_test());
    assert_eq!(app.page_for_test(), 2);
    assert!(!app.older_for_test());
    assert!(app.newer_for_test());
    assert_eq!(app.page_for_test(), 1);
    assert!(app.back_for_test());
    assert!(!app.back_for_test());
}

#[test]
fn category_and_list_screens_render() {
    let mut app = LogsApp::new();
    let mut data = vec![0u8; MonoSurface::buffer_len(410, 502)];
    let mut surface = MonoSurface::new(410, 502, &mut data).unwrap();
    let viewport = Region::full(410, 502);

    let categories = app
        .render_frame(&mut surface, &Theme::WATCH, viewport)
        .unwrap();
    assert!(categories.is_full());

    app.begin_load(LogCategory::Wakes);
    app.ingest(wake(1_000));
    app.finish_load();
    let list = app
        .render_frame(&mut surface, &Theme::WATCH, viewport)
        .unwrap();
    assert!(!list.is_empty());
}

#[test]
fn logs_app_stays_within_the_watch_stack_budget() {
    let size = core::mem::size_of::<LogsApp>();
    assert!(size <= 8 * 1024, "LogsApp is {size} bytes");
}
