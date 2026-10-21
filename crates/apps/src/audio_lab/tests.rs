//! Host tests for the Audio Lab app.

use super::app::*;
use super::view;
use common::{Mode, Region};
use ui::{GraySurface, Theme};
use voice::lab::{LabCommand, LabEvent, LabIntent, Meter};

use super::*;

#[test]
fn gain_and_volume_wrap() {
    assert_eq!(next_gain(0), 3);
    assert_eq!(next_gain(30), 0);
    assert_eq!(next_volume(90), 100);
    assert_eq!(next_volume(100), 0);
}

#[test]
fn record_intent_uses_the_selected_duration() {
    let mut app = AudioLab::new(Mode::Desk);
    app.activate(view::DURATION_HIT_ID);
    assert_eq!(app.record_ms, 10_000);
    app.activate(view::RECORD_HIT_ID);
    assert_eq!(
        app.take_intent(),
        Some(LabIntent::Command(LabCommand::Record {
            duration_ms: 10_000,
            tone_hz: 0
        }))
    );
}

#[test]
fn desk_render_registers_action_hits() {
    use common::Region;
    use ui::{GraySurface, Theme};
    let mut app = AudioLab::new(Mode::Desk);
    let mut data = vec![0u8; GraySurface::buffer_len(410, 502)];
    let mut surface = GraySurface::new(410, 502, &mut data).unwrap();
    let damage = app
        .render_frame(&mut surface, &Theme::DESK, Region::full(410, 502))
        .unwrap();
    assert!(damage.is_full());
    assert!(app
        .hits
        .targets()
        .iter()
        .any(|target| target.id == view::RECORD_HIT_ID));
    assert!(app
        .hits
        .targets()
        .iter()
        .any(|target| target.id == view::TABS_HIT_ID));
}

#[test]
fn tone_buttons_queue_playback() {
    let mut app = AudioLab::new(Mode::Desk);
    app.activate(view::TONE_1K_HIT_ID);
    assert_eq!(
        app.take_intent(),
        Some(LabIntent::Command(LabCommand::PlayTone {
            frequency_hz: 1_000,
            duration_ms: 1_000
        }))
    );
}

#[test]
fn recording_event_updates_meters() {
    let mut app = AudioLab::new(Mode::Desk);
    app.ingest(LabEvent::Recording {
        elapsed_ms: 500,
        mic1: Meter {
            peak: 16_384,
            rms: 8_000,
            clipped: false,
        },
        mic2: Meter::SILENT,
    });
    assert!(app.is_active());
    assert_eq!(app.mic1.peak, 16_384);
    assert_eq!(app.status_text[..1], *b"R");
}

#[test]
fn audio_lab_stays_within_the_watch_stack_budget() {
    let size = core::mem::size_of::<AudioLab>();
    assert!(size <= 8 * 1024, "AudioLab is {size} bytes");
}
