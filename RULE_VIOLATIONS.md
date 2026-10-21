# Rule Violations

Checklist generated from a Rust source audit against `AGENTS.MD`.
Scope: `crates/`, `drivers/`, `firmware/` (test files noted separately).
Docs were not used. Check items off as they are fixed.

## 1. Files over 300 lines

- [ ] `firmware/src/session.rs` — 705
- [ ] `crates/apps/src/audio_lab/app.rs` — 477
- [ ] `crates/shell/src/shell.rs` — 425
- [ ] `crates/apps/src/audio_lab/view.rs` — 422
- [ ] `crates/apps/src/settings_app/app.rs` — 390
- [ ] `firmware/src/desk/audio/analysis.rs` — 379
- [ ] `crates/apps/src/overlays/drawer.rs` — 373
- [ ] `crates/apps/src/home/state.rs` — 335
- [ ] `crates/storage/src/time_series.rs` — 333
- [ ] `firmware/src/main.rs` — 329
- [ ] `crates/apps/src/logs/app.rs` — 316
- [ ] `crates/ui/src/widgets/paint.rs` — 316
- [ ] `crates/settings/src/value.rs` — 303
- [ ] `crates/shell/tests/navigation.rs` — 361 (test)

## 2. Functions over 40 lines

- [ ] `firmware/src/main.rs:53` — `firmware` (264)
- [ ] `firmware/src/desk/audio/transport/capture.rs:20` — `record` (137)
- [ ] `firmware/src/session.rs:587` — `run_desk` (115)
- [ ] `crates/apps/src/audio_lab/view.rs:251` — `finish` (101)
- [ ] `firmware/src/session.rs:170` — `run` (100)
- [ ] `firmware/src/session.rs:353` — `run_watch` (99)
- [ ] `crates/apps/src/logs/view.rs:84` — `build_list` (90)
- [ ] `crates/apps/src/home/desk.rs:18` — `build` (89)
- [ ] `crates/apps/src/home/watch.rs:14` — `build` (82)
- [ ] `crates/apps/src/overlays/drawer.rs:150` — `build` (79)
- [ ] `firmware/src/desk/audio/transport/playback.rs:13` — `play_clip` (73)
- [ ] `crates/settings/src/codec.rs:75` — `assign` (73)
- [ ] `firmware/src/sleep_cycle.rs:54` — `enter` (65)
- [ ] `firmware/src/session.rs:512` — `service_audio` (63)
- [ ] `firmware/src/awake.rs:71` — `run` (60)
- [ ] `crates/storage/src/time_series.rs:159` — `prune_to_count` (60)
- [ ] `firmware/src/persist.rs:141` — `init` (59)
- [ ] `firmware/src/desk/mod.rs:57` — `start` (56)
- [ ] `firmware/src/desk/lab.rs:79` — `worker` (55)
- [ ] `crates/apps/src/audio_lab/app.rs:176` — `ingest` (54)
- [ ] `crates/apps/src/logs/app.rs:98` — `render_frame` (54)
- [ ] `crates/apps/src/audio_lab/app.rs:102` — `new` (53)
- [ ] `crates/apps/src/audio_lab/view.rs:141` — `build_mic` (53)
- [ ] `crates/shell/src/gesture.rs:192` — `classify` (53)
- [ ] `firmware/src/session.rs:293` — `switch` (51)
- [ ] `firmware/src/desk/audio/transport/playback.rs:98` — `play_tone` (50)
- [ ] `crates/ui/src/widgets/measure.rs:35` — `measure` (50)
- [ ] `crates/apps/src/settings_app/app.rs:103` — `render_frame` (50)
- [ ] `firmware/src/log_boot.rs:45` — `run` (48)
- [ ] `crates/apps/src/audio_lab/view.rs:85` — `build_loopback` (47)
- [ ] `crates/storage/src/kv.rs:59` — `open` (46)
- [ ] `crates/ui/src/widgets/patterns/feedback.rs:25` — `paint_error` (46)
- [ ] `crates/apps/src/overlays/drawer.rs:63` — `render_frame` (42)
- [ ] `crates/ui/src/painter/mod.rs:157` — `stroke_rect` (41)
- [ ] `crates/ui/src/widgets/paint.rs:23` — `paint_widget` (41)
- [ ] `crates/apps/src/settings_app/app.rs:220` — `build` (41)
- [ ] `crates/storage/src/time_series.rs:226` — `prune_logs` (41)

## 3. Functions over 5 parameters

- [ ] `crates/apps/src/logs/view.rs:84` — `build_list` (10)
- [ ] `crates/apps/src/settings_app/app.rs:220` — `build` (9)
- [ ] `crates/apps/src/audio_lab/format.rs:86` — `detail_text` (7)
- [ ] `crates/apps/src/audio_lab/view.rs:251` — `finish` (7)
- [ ] `crates/apps/src/overlays/drawer.rs:150` — `build` (7)
- [ ] `crates/ui/src/text/draw.rs:47` — `draw_text` (7)
- [ ] `crates/apps/src/audio_lab/view.rs:70` — `build` (6)
- [ ] `crates/apps/src/audio_lab/view.rs:85` — `build_loopback` (6)
- [ ] `crates/apps/src/audio_lab/view.rs:141` — `build_mic` (6)
- [ ] `crates/apps/src/audio_lab/view.rs:203` — `build_speaker` (6)
- [ ] `crates/apps/src/home/app.rs:51` — `new_with_formats` (6)
- [ ] `crates/common/src/datetime.rs:33` — `new` (6)
- [ ] `crates/ui/src/layout/measure/containers.rs:61` — `measure_list` (6)
- [ ] `crates/ui/src/text/draw.rs:82` — `draw_line` (6)
- [ ] `crates/ui/src/text/draw.rs:101` — `draw_glyph` (6)
- [ ] `crates/ui/src/widgets/button.rs:102` — `paint_icon_label` (6)
- [ ] `crates/ui/src/widgets/ring.rs:98` — `draw_ring` (6)
- [ ] `crates/ui/src/widgets/tabs.rs:86` — `paint_cell` (6)
- [ ] `crates/ui/src/widgets/toggle.rs:92` — `paint_switch` (6)
- [ ] `firmware/src/session.rs:137` — `boot` (6)
- [ ] `firmware/src/session.rs:293` — `switch` (6)

## 4. `println!` in production

68 occurrences across 10 files.

- [ ] `firmware/src/session.rs` — 22
- [ ] `firmware/src/main.rs` — 13
- [ ] `firmware/src/runtime.rs` — 9
- [ ] `firmware/src/desk/mod.rs` — 8
- [ ] `firmware/src/sleep_cycle.rs` — 6
- [ ] `firmware/src/boot.rs` — 3
- [ ] `firmware/src/awake.rs` — 3
- [ ] `firmware/src/persist.rs` — 2
- [ ] `firmware/src/log_boot.rs` — 1
- [ ] `firmware/src/log_view.rs` — 1

## 5. Global mutable state

- [ ] `firmware/src/log_boot.rs:23` — `static mut LOG_BUFFER`
- [ ] `firmware/src/retained.rs:10` — `static mut RETAINED`

## 6. Hardcoded pins outside `drivers`

Pin numbers are duplicated as GPIO types/tokens in firmware instead of using `drivers/src/board.rs`.

- [ ] `firmware/src/main.rs:178-218,312`
- [ ] `firmware/src/display/mod.rs:41-53`
- [ ] `firmware/src/sleep_cycle.rs:46`
- [ ] `firmware/src/desk/mod.rs:35-45`
- [ ] `firmware/src/desk/audio/transport/mod.rs:53-63`
- [ ] `firmware/src/runtime.rs:41-42`
- [ ] `firmware/src/boot.rs:11,58`

## 7. Allocation in render path

- [ ] `firmware/src/present.rs:98-108` — `render_gray` does `try_reserve_exact`/`resize` on a `Vec`.

## 8. Missing doc comments

- [ ] 162 public items lack doc comments (92 in generated `es7210/regs.rs` and `es8311/regs.rs`, 70 hand-written).
- [ ] `crates/ui/src/assets/icons.rs` — missing module doc (generated).
- [ ] `crates/ui/src/assets/mod.rs` — missing module doc (generated).
- [ ] `drivers/src/lib.rs:10-23` — public modules lack doc comments.
- [ ] `crates/apps/src/lib.rs:11` — `pub mod home`.
- [ ] `crates/voice/src/lib.rs:9` — `pub mod lab`.
- [ ] `firmware/src/desk/mod.rs:8` — `pub mod lab`.
- [ ] `firmware/src/display/mod.rs:61` — `pub enum BringUpError`.

## 9. Feature flags documentation

- [ ] Feature flags exist in `firmware`, `drivers`, and several crates, but the workspace `Cargo.toml` has no `[features]` documentation section.

## 10. Clippy not clean

- [ ] `crates/apps/src/logs/format.rs:8` — `needless_lifetimes`
- [ ] `crates/apps/src/logs/view.rs:84` — `too_many_arguments` (10/7)
- [ ] `crates/apps/src/settings_app/app.rs:220` — `too_many_arguments` (9/7)

## 11. Minor

- [ ] `firmware/src/metrics.rs:192` — `unsafe extern "C"` lacks a `SAFETY:` justification comment.

## Passing checks

- [x] No production `unwrap`/`expect`.
- [x] No production `panic!`, `todo!`, or `unimplemented!`.
- [x] No `std` in watch code paths.
- [x] `cargo fmt --all -- --check` clean.
- [x] All `unsafe` blocks except the extern above have safety comments.
- [x] No commented-out code found.
