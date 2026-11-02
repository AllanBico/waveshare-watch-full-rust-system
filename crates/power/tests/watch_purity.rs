//! Watch-mode purity checks over the firmware boot sources.
//!
//! The firmware binary is `no_std` and cannot run in the host test harness.
//! These tests pin the boot contract that the host cannot execute: watch boot
//! never spawns voice/sync/audio work, keeps its wake path heap-free, and
//! carries no unconditional bring-up logging.

const MAIN: &str = include_str!("../../../firmware/src/main.rs");
const BOOT: &str = include_str!("../../../firmware/src/boot.rs");
const RUNTIME: &str = include_str!("../../../firmware/src/runtime.rs");
const SESSION: &str = include_str!("../../../firmware/src/session.rs");
const SLEEP: &str = include_str!("../../../firmware/src/sleep_cycle.rs");
const CPU1: &str = include_str!("../../../firmware/src/desk/cpu1.rs");

#[test]
fn watch_boot_spawns_no_voice_sync_audio_or_radio_work() {
    for source in [MAIN, BOOT, RUNTIME, SESSION, SLEEP] {
        for forbidden in [
            "mod audio;",
            "mod voice;",
            "mod sync;",
            "audio::",
            "voice::",
            "wifi::task",
            "ble::task",
            "esp_radio::",
        ] {
            assert!(
                !source.contains(forbidden),
                "watch boot source must not reference {forbidden:?}"
            );
        }
    }
    for source in [MAIN, SESSION] {
        assert!(!source.contains("sync::"), "no sync work in watch boot");
    }
}

#[test]
fn watch_boot_never_starts_a_radio() {
    assert!(MAIN.contains("if mode == Mode::Desk"));
    assert!(!MAIN.contains("radio.wifi("));
    assert!(!MAIN.contains("radio.ble("));
    assert!(SESSION.contains("fn run_desk"));
    assert!(SESSION.contains("service_connectivity(shell, radio)"));
}

#[test]
fn the_wake_path_does_not_allocate() {
    for source in [MAIN, BOOT, RUNTIME, SESSION, SLEEP] {
        for forbidden in ["Box::new", "Box::leak", "vec!"] {
            assert!(
                !source.contains(forbidden),
                "wake path must not allocate with {forbidden:?}"
            );
        }
    }
}

#[test]
fn serial_boot_output_is_mode_gated() {
    assert!(MAIN.contains("let log = plan.logging;"));
    assert!(MAIN.contains("report_boot(mode, psram_bytes, log)"));
    assert!(RUNTIME.contains("pub fn report_boot(mode: Mode, psram_bytes: usize, log: bool)"));
}

#[test]
fn watch_boot_never_starts_the_audio_core() {
    assert!(MAIN.contains("if mode == Mode::Desk"));
    assert!(MAIN.contains("desk::start(desk_inputs, &mut core1)"));
    assert!(!MAIN.contains("start_audio"));
    assert!(SESSION.contains("core1.park();"));
    assert!(SESSION.contains("audio.shutdown().await;"));
    assert!(CPU1.contains("start_app_core"));
    assert!(CPU1.contains("InterruptExecutor"));
    assert!(CPU1.contains("park_core"));
}
