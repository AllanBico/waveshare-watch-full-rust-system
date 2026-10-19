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

#[test]
fn watch_boot_spawns_no_voice_sync_audio_or_radio_work() {
    for source in [MAIN, BOOT, RUNTIME, SESSION, SLEEP] {
        for forbidden in [
            "mod audio;",
            "mod voice;",
            "mod sync;",
            "audio::",
            "voice::",
            ".spawn(",
            "wifi",
            "bluetooth",
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
