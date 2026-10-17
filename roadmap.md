## Phase 0: Foundation

**Goal:** Project skeleton compiles, flashes, and runs on hardware.

### 0.1 Workspace Setup
- Create workspace with empty crates: `common`, `hal`, `drivers`, `power`, `storage`, `ui`, `apps`, `voice`, `sync`, `logging`, `firmware`.
- `core` is named `common` because a crate named `core` shadows the `core` prelude.
- Set up feature flags on the `firmware` package: `logging`, `trace`, `simulator`. Mode is runtime, not a feature. (A virtual workspace root cannot declare `[features]`.)
- Pin the `esp` toolchain in `rust-toolchain.toml`.
- Configure `.cargo/config.toml` for the ESP32-S3 target, with `build-std = ["core"]` since the esp toolchain ships no prebuilt `core` for Xtensa.
- Keep `firmware` a `no_std` library for now; it becomes a binary in Phase 0.2.

**Test:** `cargo build` succeeds with empty crates for the ESP32-S3 target. `cargo fmt --check` and `cargo clippy` are clean.

### 0.2 Hardware Bring-Up (Done)
- Smoke test the toolchain: `espflash board-info` connects and reads the chip.
- Turn `firmware` into a `#[esp_rtos::main]` binary with an `esp-backtrace` panic handler and `esp_app_desc!`.
- Bring up `esp-rtos` for the executor and the embassy time driver.
- On boot, print chip model, revision, CPU clock, MAC, PSRAM size, and reset reason over USB serial (`esp-println`).
- Run a periodic heartbeat with uptime on an embassy timer. No busy-waiting.
- Flash with `espflash` through the configured runner.

**Test:** Serial prints `chip: esp32s3 rev v0.2`, `cpu: 160 MHz`, the MAC, `psram: 8388608 bytes` (8 MB, matching the S3R8), a reset reason, and a 1 Hz heartbeat with rising uptime. `cargo fmt --check` and `cargo clippy` are clean. Flash size (32 MB) is reported by `espflash`, not at runtime. There is no user LED; bring-up is verified over serial.

**Note:** esp-rtos pins `esp-hal` to 1.0.0, so the crate APIs are the 1.0.0 ones (e.g. `Efuse::mac_address`, `psram::psram_raw_parts`).

### 0.3 Core Types (Done)
- `Mode`, `Error` (stable numeric codes), `Event`, `WakeReason`, `Timestamp`, and a schema-versioned `LogEntry` in `common`.
- Optional `defmt` feature for `Format` derives. No hardware dependencies.
- Crates use `#![cfg_attr(not(test), no_std)]` and `#![forbid(unsafe_code)]` so they can be host-tested.

**Test:** 17 unit tests pass (construction, conversion, edge cases, and error-code stability/uniqueness). `cargo fmt --check` and `cargo clippy` are clean. Host tests run via `./tools/test-host.sh`.

---

## Phase 1: Power Foundation

**Goal:** AXP2101 controlled, VBUS detection works, deep sleep enters and exits.

### 1.1 AXP2101 Driver
- I2C communication with AXP2101.
- Read VBUS status, battery voltage, battery percentage.
- Set rail voltages, enable/disable rails.

**Test:** Read battery voltage on hardware. Matches multimeter within tolerance.

### 1.2 Power Manager Skeleton
- `PowerManager` struct owns AXP2101 handle.
- `request_awake(reason, budget_ms)` and `enter_sleep()`.
- Rail control API.

**Test:** Manually cut display rail. Display goes dark.

### 1.3 Deep Sleep Entry/Exit
- Configure RTC timer wakeup.
- Configure GPIO wakeup (button).
- Enter deep sleep, wake on timer, log wake reason.

**Test:** Watch sleeps for 10 seconds, wakes, prints "woke by timer".

### 1.4 VBUS Detection
- Poll VBUS on boot and on wake.
- Set `Mode::Watch` or `Mode::Desk` accordingly.

**Test:** Plug USB, mode switches to Desk. Unplug, mode switches to Watch.

### 1.5 PCF85063A RTC Driver
- I2C communication with PCF85063A.
- Read and write time and date (BCD registers).
- Expose it through the `Time` HAL trait.
- `init` configures 24-hour mode and CLKOUT without a soft reset, so the time
  survives deep sleep and reboots. A soft reset on every boot would clear the
  time registers.

**Test:** Set the time, read it back, and confirm it advances. Timestamps are
available for logs and survive a deep sleep without being reset.

---

## Phase 2: Storage Foundation

**Goal:** TicKV works over a dedicated flash partition, the time-series layer
works, and data survives reboot.

### 2.1 TicKV Integration (Done)
- Dedicated `kv` partition (256 KiB, data subtype `undefined`) plus a 2 MiB
  `logs` partition reserved for 2.2. The app partition is 8 MiB. The layout
  lives in `partitions.csv` and is written by the `espflash` runner.
- `drivers::flash` owns the single `esp-storage` handle, reads the partition
  table once, and exposes labelled partitions as `NorFlash` regions. Do not use
  `embedded_storage::Storage::write`: it erases a whole sector per call.
- `storage` wraps `tickv` 2.0 with a 4096-byte region. The adapter uses
  `NorFlash` read/write/erase and a 4-byte read-modify-write path for TicKV's
  unaligned writes.
- Keys use a frozen FNV-1a + SplitMix64 hash, including the TicKV super key.
  Changing the hash orphans every key, so it is part of the storage format.
- Boot probes the super key. Only a fully blank partition may be provisioned,
  and only in desk mode. A missing super key on non-blank flash is corruption,
  never permission to erase. Watch mode opens existing data read-only and
  reports `KV NONE` for an unprovisioned partition.
- `set` is invalidate-then-append because TicKV rejects duplicate hashes.
  `delete` invalidates; GC runs on explicit request and on a `FlashFull` retry.
  The schema version is stored under `meta/schema` and checked on open.
- Trace builds show a display viewer with the store status, a persisted boot
  counter, and the schema version or an error code. The display is a viewer;
  the store is the log. Production boots never write a test key. Trace
  selftests may increment the counter every boot to prove persistence without
  serial.

**Test:** Host tests cover provisioning policy, write/reboot/read, overwrite,
delete, unaligned values, maximum value, small buffers, corruption without
erase, schema mismatch, full/GC, and flash error mapping. Hardware verified:
the trace viewer shows `KV OK`, `VER 01`, and a boot counter that increments
across power cycles. `cargo fmt` and `cargo clippy` are clean.

### 2.2 Time-Series Layer (Done)
- TicKV has no key enumeration, so `TimeSeries` keeps explicit `log/latest`
  and `log/oldest` index keys. Entries use fixed-width `log/{sequence}` keys.
- `append_log(entry)` encodes the versioned `LogEntry` (binary, schema 1),
  writes the entry before advancing `latest`, and returns the sequence. A
  crash between the two writes leaves an orphan that the next append replaces.
- `read_logs_since(checkpoint)` streams decoded entries and returns the last
  delivered sequence. Missing/pruned sequences are skipped. The callback can
  stop early; the returned checkpoint then points at the last accepted entry.
- `prune_logs(before_timestamp)` invalidates a timestamp prefix, advances
  `oldest`, and skips already-missing entries. GC reclaims space explicitly or
  on a `FlashFull` retry. Pruning assumes entries are in timestamp order.
- Open and format the `logs` partition only in desk mode. Watch mode never
  formats flash. The logs store may be force-formatted because log data is
  disposable; the settings `kv` partition remains strict and never force-formats.
- The display is the storage viewer during Phase 2, not the clock. The viewer
  is compiled by default; the non-trace build shows status, `VER`, and
  `LOG NONE`. Trace firmware additionally appends one entry per boot and shows
  the latest sequence as `LOG nnnn`. The display remains a viewer; the store is
  the log.

**Test:** Host tests append 100 entries, read them back, and verify order and
content; they also cover checkpoints, early stop, prefix/all pruning, reopen,
sequence gaps, corrupt records, and schema mismatches. On hardware, the first
trace desk boot provisions the 2 MiB `logs` partition and the viewer should
show an incrementing `LOG` sequence. The log partition is not touched on production watch boots.

### 2.3 Logging Crate (Done)
- `logging::Ring` is a 96-slot retained ring in RTC slow memory
  (`#[esp_hal::ram(unstable(rtc_slow, persistent))]`). It stores encoded
  `LogEntry` records with a magic/version header, head, count, and dropped
  counters. Normal SRAM does not survive deep sleep; this section does.
- The ring records wake reasons and battery samples every boot. A timestamp
  comes from the PCF85063A through the `Time` HAL.
- The `log!` macro expands to nothing unless the caller's `logging` feature is
  enabled. Core battery/wake telemetry is always compiled; the feature gates
  optional event logs.
- Flush policy: desk builds flush every boot; watch builds (including trace)
  flush at 32 retained records (about 16 wakes) and only above the write
  voltage. Never write flash on every wake in watch mode.
- The `logs` partition is force-formatted only in desk mode because logs are
  disposable. Watch mode opens it read-only and never formats.
- Unexpected log open/flush failures are recorded as `Event::Error` entries for
  the next successful flush. Expected `NotReady` in unprovisioned watch mode is
  not recorded.
- The display viewer shows `BUF`/`DRP` counts and the latest persisted `LOG`
  sequence. It remains a viewer; the retained ring and flash are the log.

**Test:** Six ring tests cover initialization from garbage, push/peek/pop,
reopen retention, wraparound/drop accounting, clear, and corrupt slots. Host
tests verify the codec and time-series. On hardware: unplug USB to enter watch mode;
wakes should show `BUF` growing by two records; at the threshold they flush and
`LOG` advances. Desk boots flush immediately, so `BUF` stays zero while USB is
connected.

### 2.4 Battery Voltage Logging (Done, combined with 2.3)
- Every boot reads AXP2101 battery voltage once and appends an
  `Event::Battery(Millivolts)` record to the retained ring with the RTC
  timestamp. Wake reason is recorded in the same pass.
- Samples flush with the same policy as 2.3: desk every boot, watch every 32
  records. No flash write happens per wake.
- `Battery` and `Error` are additive event tags under the existing log schema
  version 1; existing wake/mode records remain readable.
- The viewer shows the current `BATT` sample, the retained `BUF` count, and the
  latest persisted `LOG` sequence. It is a viewer, not the log.

**Test:** Host ring/codec tests cover battery records and retention. On
hardware: after ten watch wakes the viewer should show `BUF 20` (wake plus
battery records); plugging USB (desk) flushes them and `LOG` advances with
`BUF` returning to zero. No flash write occurs on the ten watch wakes.

## Phase 3: Display Foundation

**Goal:** CO5300 displays pixels, monochrome mode works, partial refresh works.

### 3.1 CO5300 Driver
- QSPI communication.
- Initialize display.
- Write full framebuffer.

**Test:** Display solid color on hardware.

### 3.2 Partial Refresh
- Dirty rect support.
- Only push changed regions.

**Test:** Draw a small square. Verify only that region updates.

### 3.3 Monochrome Mode
- 1-bit framebuffer.
- Black and white only.
- Pixel budget tracking.

**Test:** Draw a digit test pattern. Count lit pixels. Under 5%.

### 3.4 Display Power Control
- The AMOLED has no dedicated rail. Its power pins tie to VCC3V3 (DCDC1, shared
  with the SoC) and its enable `DSI_PWR_EN` is pulled up to ALDO2 (VL2) and
  otherwise driven by a GPIO. See `docs/hardware/axp2101-reference.md` and
  `docs/hardware/board-pins.md`. The earlier "cut the display rail" framing was
  wrong.
- Put the panel into sleep (SLPIN) before cutting the enable; never drive QSPI
  into an unpowered panel.
- Cut the display enable through `power`; ALDO2 is the sole enable (the
  schematic shows no enable GPIO). This means removing ALDO2 from `SLEEP_PLAN`,
  which currently keeps it on.
- On wake: restore the enable, re-run the full init sequence (the panel lost
  power, so SLPOUT alone is not enough), and re-flush the retained framebuffer.

**Test:** Sleep, measure current. Panel sleep plus the display enable off saves
measurable power.

---

## Phase 4: Input Foundation

**Goal:** Touch and IMU work, wake-on-motion works, button works.

### 4.1 FT3168 Touch Driver
- I2C communication.
- Read touch coordinates.
- Sleep mode command.

**Test:** Touch screen, read coordinates.

### 4.2 QMI8658 IMU Driver
- I2C communication.
- Read accelerometer.
- Configure Wake-on-Motion threshold.

**Test:** Tap watch, read accel spike.

### 4.3 Wake-on-Motion
- IMU INT pin wakes ESP32 from deep sleep.
- Wake reason logged.

**Test:** Sleep, tap watch, wakes, logs "woke by motion".

### 4.4 Button Wake
- GPIO wake from deep sleep.
- Debounce.

**Test:** Sleep, press button, wakes, logs "woke by button".

### 4.5 Wake Verification
- On motion wake, read accelerometer.
- Check orientation.
- Decide: show display or go back to sleep.

**Test:** Walk with watch. Verify no false screen-on. Raise wrist. Verify screen-on.

---

## Phase 5: Monochrome UI Foundation

**Goal:** Both modes render through one monochrome engine, the shell owns
navigation and overlays, and Home is the base screen.

### 5.1 Luminance Core
- Delete the temporary `clock`, `font`, `mono`, `debug`, and firmware viewer.
- Define `Theme` structs `WATCH` (1-bit) and `DESK` (8-bit grayscale).
- Define tone, type, spacing, radius, border, and motion tokens.
- Replace color policy with pixel mode, animations, and pixel budget.
- Implement `MonoSurface` and `GraySurface` with fill, stroke, rounded rect,
  line, and clip.
- Add the host golden-frame harness.

**Test:** Fills and clips in both surfaces; no out-of-bounds; token snapshots.

### 5.2 Text & Assets (depends on 5.1)
- Build `tools/assetgen` with `assets/manifest.toml`.
- Generate 4-bit font masks from an OFL Google Font at exact token sizes.
- Generate a tabular numeric subset for clock and data.
- Generate icon coverage masks and an `IconId` enum from SVG sources.
- Implement text measure, wrap, ellipsis, align, and a fixed-size measurement cache.
- Watch thresholds coverage to 1-bit; desk blends into Gray8.
- Report asset sizes; fail assets over 64 KB without approval.

**Test:** Glyph metrics, cache hits and misses, missing glyph fallback, asset-size
gate, text golden frames.

### 5.3 Layout Engine (depends on 5.2)
- Implement integer constraints and `Fixed`, `Fill`, `Percent`, `Fit`.
- Add `Row`, `Column`, `Stack`, `Grid`, `List`, `Scroll`, and `Spacer`.
- Run measure then arrange; no layout during paint.
- Integrate safe area and cached text measurement.
- Enforce max depth 32.
- Lay out only visible rows in fixed-height lists.

**Test:** Deterministic layout snapshots, grow/shrink, depth cap, overflow and
clip, scroll viewport.

### 5.4 Scene, Diff & Damage (depends on 5.3)
- Build a fixed-capacity node arena with stable `WidgetId`s.
- Implement build, layout, diff, damage, paint, and flush passes.
- Store a previous-frame snapshot of id, rect, and fingerprint.
- Dirty = old rect union new rect; merge, coalesce, and cap the rect count.
- Collapse a moving container to one dirty rect.
- Repair damage inside one framebuffer, clipped to the damage rect.
- Expand Mono bits and convert Gray8 through a luma LUT.
- Sync flushes to TE; cap desk at 60 fps.
- Full redraw only on boot, mode switch, or theme switch.

**Test:** Move, insert, and remove produce expected dirty rects; no full-screen
damage; watch path allocates zero; dirty-rect cap holds; theme switch redraws all.

### 5.5 Components & Patterns (depends on 5.2-5.4)
- Implement the common widget behavior over enum-backed widget kinds.
- Add Button, IconButton, ListItem, Toggle, Slider, Checkbox, Radio, Input,
  Progress, Badge, Dialog, Toast, ContextMenu, Tabs, and StatusIndicator.
- Support normal, focused, pressed, disabled, error, and selected states.
- Draw a visible focus ring and register hit rects.
- Add ScreenScaffold, EmptyState, LoadingState, ErrorState, and Confirm patterns.

**Test:** State rendering, pure props, focus traversal, hit testing, no app logic,
and per-component watch pixel budget.

### 5.6 Shell & Navigation (depends on 5.5)
- Define the `App` trait and `AppContext`; the registry is implemented by firmware.
- Add app lifecycle: enter, exit, tick, action.
- Add layers: app, app drawer, quick settings, dialog, toast.
- Add the route stack and abstract actions.
- Enforce back order: dialog, overlay, app route, home.
- Add the edge-gesture state machine with slop, threshold, and velocity.
- Snap in watch; animate in desk.
- Block input behind modal dialogs; toasts never take focus.
- Move status ownership into the shell.

**Test:** Back order, gesture traces, modal blocking, lifecycle, app isolation.

### 5.7 Home + System UI (depends on 5.6)
- Pin `Home` as the base app for both modes; the clock is a complication.
- Watch Home: light clock, date, battery, minimal chrome.
- Desk Home: grayscale dashboard using the same components.
- Open the app drawer from the bottom edge; use three columns.
- Open quick settings from the top edge; use two columns.
- Switch theme and mode at runtime without reboot.

**Test:** Boot to Home, drawer and quick-settings gestures, theme switch, mode
switch, watch under 5% lit pixels, desk 60 fps budget, no full flush after boot.

---

## Phase 6: Watch Mode Integration

**Goal:** Full watch mode works end-to-end. Battery lasts measurable time.

### 6.1 Watch Mode Entry
- Boot into watch mode.
- Initialize minimum peripherals.
- Enter deep sleep.

**Test:** Watch boots, shows Home, sleeps.

### 6.2 Wake Cycle
- Wake on motion, button, or timer.
- Verify wake reason.
- Show Home if appropriate.
- Return to sleep.

**Test:** 100 wake cycles. No crashes. Battery voltage logged each time.

### 6.3 Alarm Support
- Store alarms in TicKV.
- Set RTC timer to next alarm before sleep.
- Vibrate on alarm.

**Test:** Set alarm 1 minute ahead. Watch vibrates on time.

### 6.4 Battery Life Baseline
- Full charge.
- Normal usage pattern.
- Measure time to shutdown.

**Test:** Record a first battery-life baseline. This is a goal, not a gate; later phases must not regress it.

---

## Phase 7: Desk Mode Foundation

**Goal:** USB detection switches to desk mode. Full UI renders. No voice yet.

### 7.1 Desk Mode Entry
- Detect VBUS in `main`, dispatch to `desk_init` (same binary as watch mode).
- Initialize full peripherals.
- Flip the render policy to desk mode.

**Test:** Plug USB, UI switches to desk monochrome theme.

### 7.2 Mode Switch
- Graceful switch from watch to desk.
- Graceful switch from desk to watch.
- State preserved across switch.

**Test:** Plug and unplug USB 10 times. No crashes. State intact.

### 7.3 Full UI
- All apps available.
- Animations enabled.
- Smooth transitions.

**Test:** Navigate all apps. Animations play.

### 7.4 Settings App
- Change theme.
- Change brightness.
- View battery stats.
- View logs.

**Test:** Change setting, verify persistence after reboot.

---

## Phase 8: Voice Foundation

**Goal:** Wake word works locally. Audio streams to VPS. TTS plays back.

### 8.1 Audio Hardware
- ES8311 codec init.
- I2S microphone capture.
- I2S speaker playback.

**Test:** Record 5 seconds, play back.

### 8.2 Wake Word
- Hand-rolled `no_std` pipeline: I2S capture, `microfft` FFT, MFCC, small NN with `const` weights.
- Train the small model and embed the weights as constants.
- VAD gates the model; add a confidence threshold.

**Test:** Say wake word, detection triggers. Soft speech does not trigger. False-accept rate is acceptable.

### 8.3 Audio Streaming
- Stream PCM (or simple Rust-side compression) over WebSocket.
- No Opus on the watch; the VPS handles codecs.
- VPS performs endpointing.

**Test:** VPS receives audio, saves to file.

### 8.4 TTS Playback
- Receive PCM from the VPS (VPS decodes Opus).
- Play through speaker.
- Ring buffer for DMA.

**Test:** VPS sends TTS, watch plays it.

### 8.5 Voice Pipeline
- Wake word → stream → VPS → TTS → playback.
- End-to-end latency under 2 seconds.

**Test:** Say "Hey Watch, what time is it", hear response.

---

## Phase 9: Sync Foundation

**Goal:** Watch syncs logs and settings with VPS. Alarms sync. Checkpoints work.

### 9.1 Sync Protocol
- Define message format (CBOR or custom).
- Define checkpoint semantics.
- Define ACK handling.

**Test:** Send dummy record, receive ACK.

### 9.2 Log Sync
- Read unsynced logs from TicKV.
- Push to VPS.
- Mark as synced on ACK.

**Test:** Generate logs, sync, verify VPS has them.

### 9.3 Settings Sync
- Pull settings from VPS.
- Apply to watch.
- Push local changes.

**Test:** Change setting on VPS, watch receives it.

### 9.4 Alarm Sync
- Pull alarms from VPS.
- Store in TicKV.
- Push local alarms.

**Test:** Create alarm on VPS, watch vibrates on time.

### 9.5 Conflict Resolution
- Define precedence: VPS wins or watch wins.
- Handle duplicate records.

**Test:** Modify same setting on both, verify resolution.

---

## Phase 10: AI Integration

**Goal:** VPS node handles LLM, tools, TTS. Watch is thin client.

### 10.1 VPS Node Setup
- Rust service on VPS.
- WebSocket server.
- DeepSeek API integration.
- Gemini TTS integration.

**Test:** Send text to VPS, receive TTS audio.

### 10.2 Tool Calling
- Define tool registry in Rust.
- Implement `save_note`, `get_notes`, `set_timer`.
- LLM calls tools, executes, responds.

**Test:** Say "save note milk", verify note stored.

### 10.3 Voice Command Pipeline
- Wake word → stream → VPS → LLM → tool → TTS → playback.
- End-to-end under 3 seconds.

**Test:** Say "set timer 5 minutes", watch vibrates after 5 minutes.

### 10.4 Offline Fallback
- VPS unreachable → queue request or local command.
- User notified.

**Test:** Disconnect VPS, say command, watch handles gracefully.

---

## Phase 11: Polish & Optimization

**Goal:** Battery life maximized. UI smooth. Edge cases handled.

### 11.1 Power Audit
- Measure current in every state.
- Identify leaks.
- Fix leaks.

**Test:** Watch mode idle current under 50 µA.

### 11.2 UI Polish
- Smooth animations.
- No dropped frames.
- Theme transitions.

**Test:** 60 FPS in desk mode. No visual glitches.

### 11.3 Error Handling
- Brownout graceful shutdown.
- Flash corruption recovery.
- Network failure handling.

**Test:** Simulate brownout, verify no corruption.

### 11.4 Battery Life Retention Test
- Full charge.
- Mixed usage: 23 hours watch mode, 1 hour desk mode.
- Measure and compare against the Phase 6.4 baseline.

**Test:** No regression against the baseline, and the target goal is met if reachable.

---

## Phase 12: Stretch Goals

**Goal:** Optional features if time permits.

### 12.1 Custom Case
- 3D printed case for larger battery.
- Speaker removal.

### 12.2 OTA Updates
- `edge-ota` integration.
- A/B partition.
- Rollback on failure.

### 12.3 Additional Apps
- Stopwatch.
- Timer.
- Step counter.
- Heart rate (if sensor added).

### 12.4 Advanced Voice
- Local command recognition (10 commands).
- Multi-turn conversation.
- Context awareness.

---

## Testing Discipline

- Every phase has a testable deliverable.
- No phase advances until deliverable is verified.
- Hardware tests on real device.
- Simulator tests on desktop.
- Power tests with multimeter or INA219.
- Battery figures are goals and baselines, not hard gates. Never regress a measured baseline.
- Mode-switch transitions are tested both directions, repeatedly.
- Watch-mode purity is tested: no voice/sync task exists and the mic rail is off.
- Logs reviewed after every phase.
- Host unit tests: `./tools/test-host.sh`. It uses the stable toolchain, because the esp toolchain rebuilds `core` via `build-std` and clashes with the host's prebuilt std. A cargo alias cannot switch toolchains.

---

## Order Rationale

- Power first because everything depends on it.
- Storage second because logs depend on it.
- Display third because UI depends on it.
- Input fourth because wake depends on it.
- UI fifth because apps depend on it.
- Watch mode sixth because it's the primary use case.
- Desk mode seventh because it unlocks everything else.
- Voice eighth because it needs desk mode.
- Sync ninth because it needs storage and voice.
- AI tenth because it needs voice and sync.
- Polish last because it needs everything.
