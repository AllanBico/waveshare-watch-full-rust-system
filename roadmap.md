## Phase 0: Foundation

**Goal:** Project skeleton compiles, flashes, and runs on hardware.

### 0.1 Workspace Setup
- Create workspace with empty crates: `common`, `hal`, `drivers`, `power`, `storage`, `ui`, `apps`, `voice`, `sync`, `logging`, `firmware`.
- `core` is named `common` because a crate named `core` shadows the `core` prelude.
- Set up feature flags on the `firmware` package: `logging`, `trace`, `simulator`. Mode is runtime, not a feature. (A virtual workspace root cannot declare `[features]`.) The `simulator` flag is retired in Phase 6.1.
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

## Phase 6: Firmware Spine

**Goal:** One real firmware with one boot chain, a working shell, blank Home, and
live mode switching. Preview leaves the build. No mode-specific UI yet.

### 6.1 Preview Retirement & First Light
- Tag the preview source `preview-reference`; keep it as reference, out of the
  build.
- Remove the preview module from the firmware build and boot path.
- Remove `Preview` from the `apps` compiled API; keep `home` and `format`.
- Drop the simulator: remove the `simulator` feature, the `.cargo/config.toml`
  comment, and the simulator references in the docs.
- Paint one line (`WATCH`/`DESK`) directly through the real surface and flush
  path, with no shell and no apps.
- Report boot-stage failures visibly: a line on the panel when possible, serial
  detail always.

**Test:** Firmware builds with no preview symbols; the label paints on both
mono and gray surfaces; a failed boot is visible, never a silent halt.

### 6.2 One Boot Chain
- Use one boot sequence with explicit mode hooks, not duplicated `watch_init`
  and `desk_init` bodies.
- Order: PMIC -> RTC -> storage/settings -> VBUS mode decision -> clock/PSRAM
  hooks -> peripherals -> shell.
- Boot both modes at 80 MHz. PSRAM is a desk-only hook; watch never initializes
  it.
- Watch init: PMIC, RTC, I2C, IMU, display, mono surface.
- Desk init: the watch set plus PSRAM, touch, and the gray surface. No audio and
  no radios yet.
- Load one settings snapshot before wake configuration; no scattered reads.

**Test:** Both modes reach the same spine; no duplicated boot bodies; no PSRAM
access in watch; cold boot succeeds in both modes.

### 6.3 Shell, Registry, Blank Home
- Implement `AppRegistry` in firmware and boot the shell with a firmware-owned
  debug Home that shows only the mode label.
- Wire lifecycle (`enter`, `exit`, `tick`, `action`), the route stack, status
  ownership, and minimal overlay blocking.
- Keep the label out of `apps::Home`, so no app branches on `Mode`.
- Register only the firmware-owned `DebugHome` in both modes; the real app set
  begins in 7.1 and 8.3. Registry divergence is a declared seam, not implemented
  yet.

**Test:** Lifecycle ordering, registry resolution, back order, and modal
blocking; blank Home renders in both policies; the label changes with mode; no
`if mode` in shell or apps; watch render allocates zero.

### 6.4 Settings -> Boot Policy
- Derive a boot policy from settings: `wake_on_motion`, `motion_sensitivity`,
  `sleep_timeout_secs`, and `haptics`.
- Clamp safety limits. Settings tune behavior; they never remove hard caps.
- Classify effects: live (brightness, theme), next sleep (wake and sleep
  settings).
- Leave `power_saving`, `always_on`, and screen timeout unwired, with explicit
  semantics documented.
- Keep alarms in storage state, not settings.

**Test:** Settings are honored on cold boot and after a mode switch; thresholds
clamp; schema and persistence tests pass.

### 6.5 Mode Switch & VBUS Wake
- Hardware gate first: confirm `EXIO5`/`AXP_IRQ` reaches an RTC-capable GPIO and
  stays powered in deep sleep.
- Gate result: the repository does not map `EXIO5` to an ESP GPIO, so the IRQ
  path is not implemented. Fall back to motion/button/timer wake sampling VBUS.
  Do not add periodic polling in watch.
- If the pin is later confirmed, add the AXP VBUS-insert IRQ as a third Ext1
  source and wake into desk on insert.
- Poll VBUS for removal in desk; desk is full power and pays no power cost.
- Switch gracefully both ways: flush, finish the operation, apply the rail plan
  and render policy, re-enter the active app, and preserve state.
- Add retained wake counters and boot/wake diagnostics.

**Test:** 10 switch cycles while awake preserve state; sleep on battery then plug
USB wakes into desk (or uses the documented fallback); unplug switches to watch
and sleeps; no VBUS polling in watch; rails match the plan after every switch.

### 6.6 Spine Gate
- Record the boot, switch, sleep, and wake checklist in `docs/spine-gate.md`.
- Run the automated checks and the on-device rows before starting Phase 7.
- Temporary bring-up logging is still allowed; removing it is a watch-phase
  task.

**Test:** Both modes boot to a labeled blank Home; switching works while awake
and from sleep; preview is absent from the binary; no mode forks in the spine;
settings are honored at boot.

---

## Phase 7: Watch Mode

**Goal:** Real watch mode end-to-end: Home, minimal init, wake cycles, alarms,
ultra-low power, and a battery baseline.

### 7.1 Watch Home
- Port the watch Home from preview as inspiration, not by copying its structure:
  clock, date, battery, minimal chrome.
- Home becomes a real `App`; the debug Home is retired.
- Keep the watch-only app set minimal; no drawer or quick settings unless Home
  needs them.

**Test:** Watch Home renders in 1-bit; under 5% lit pixels on idle; no
allocation; golden frames.

### 7.2 Watch Boot & Purity
- Boot straight to watch Home on battery.
- Initialize only watch-needed peripherals; keep PSRAM uninitialized and radios,
  audio, and the mic rail off.
- Remove bring-up serial logging from watch; keep retained counters only.
- Enforce watch purity in tests: no voice or sync task, no mic rail, no heap in
  the render or wake paths.

**Test:** Watch boots, shows Home, and sleeps; purity tests pass; no logging in
watch.

### 7.3 Wake Cycle
- Wake on motion, button, or alarm; verify and record the reason.
- Show Home when appropriate, honor the awake budget, run the pre-sleep
  checklist, and return to deep sleep.
- Apply wake and sleep settings before each sleep.

**Test:** 100 wake cycles with no crashes; battery voltage recorded each cycle;
budget expiry forces sleep; the motion threshold respects settings.

### 7.4 Alarms
- Store alarms in TicKV.
- Set the next RTC wake to the next alarm, or omit the timer when none is set.
- Vibrate on alarm using the haptics setting.
- Handle missed alarms after a long sleep.

**Test:** An alarm one minute ahead vibrates on time; alarms persist across
reboot; the next-wake calculation is correct.

### 7.5 Watch Input & Settings
- Limit touch and gestures to what watch Home needs.
- Wire brightness, the watch theme variant, time format, and date format live.
- Fix timeout semantics: screen timeout is desk-only; deep sleep is the watch
  timeout.

**Test:** Settings apply live or at next sleep as classified; persistence
survives reboot.

### 7.6 Battery Life Baseline
- Measure current in sleep, idle-awake, and render states.
- Run a normal usage pattern from full charge to shutdown.
- Record the first baseline for later phases.

**Test:** Baseline recorded; measured states documented; later phases must not
regress it.

**Phase gate:** Watch is pure, ultra-low-power, wakes correctly, alarms work,
and a baseline exists.

---

## Phase 8: Desk Mode

**Goal:** Full-power desk mode on VBUS, full UI, animations, settings, logs, and
no power saving.

### 8.1 Full Desk Init
- Desk boot brings up the full peripheral set: PSRAM, touch, and display; audio
  as available. Radios remain for the sync phase.
- Raise the CPU to 240 MHz if runtime re-clocking is clean; otherwise document
  80 MHz with a follow-up.
- Desk stays awake while VBUS is present; no watch power policies apply.

**Test:** Plug USB and get a desk boot with full init; unplug and switch
gracefully to watch; no watch power policy is active in desk.

### 8.2 Switch Hardening
- Exercise watch and desk switching repeatedly with real state, not blank Home.
- Verify flush and finish ordering, state preservation, rail audit, and
  render-policy changes.
- No leaks or duplicate init across cycles.

**Test:** 10 plug/unplug cycles with no crashes, state intact, rails correct, and
no full flush except on mode or theme switch.

### 8.3 Full UI Port
- Port the shipping screens from preview inspiration: settings, app drawer,
  quick settings, overlays, dialogs, toasts, and calibration if it ships.
- Restore animations, transitions, and desk grayscale rendering.
- Implement per-mode registry divergence: desk gets the full app set.

**Test:** Navigate all apps; animations play; desk holds the 60 fps cap;
flushes are damage-only; watch behavior is unchanged.

### 8.4 Settings App
- Change theme, brightness, battery stats, and logs; persist across reboot.
- Surface which settings take effect live versus at the next sleep or mode
  switch.
- Wire the remaining safe settings; keep diagnostics, clock, and VBUS out of
  user settings.

**Test:** Changes persist after reboot; boot policy reflects them; live changes
apply immediately.

### 8.5 Preview Cleanup
- Retire the last preview-derived reference usage.
- Delete the `preview-reference` tag or branch and remove it from the docs.
- Remove bring-up diagnostics that have no product role.

**Test:** The repository contains no preview build path or dead references; the
docs match reality.

### 8.6 Desk Gate
- Full desk UI, graceful switching, settings persistence, and no power saving
  while on VBUS.
- Voice, sync, and AI remain out of scope for this phase.

**Test:** The full desk experience works end-to-end on USB; watch still boots
and sleeps correctly after desk use.

---

## Phase 9: Voice Foundation

**Goal:** Wake word works locally. Audio streams to VPS. TTS plays back.

### 9.1 Audio Hardware
- ES8311 codec init.
- I2S microphone capture.
- I2S speaker playback.

**Test:** Record 5 seconds, play back.

### 9.2 Wake Word
- Hand-rolled `no_std` pipeline: I2S capture, `microfft` FFT, MFCC, small NN with `const` weights.
- Train the small model and embed the weights as constants.
- VAD gates the model; add a confidence threshold.

**Test:** Say wake word, detection triggers. Soft speech does not trigger. False-accept rate is acceptable.

### 9.3 Audio Streaming
- Stream PCM (or simple Rust-side compression) over WebSocket.
- No Opus on the watch; the VPS handles codecs.
- VPS performs endpointing.

**Test:** VPS receives audio, saves to file.

### 9.4 TTS Playback
- Receive PCM from the VPS (VPS decodes Opus).
- Play through speaker.
- Ring buffer for DMA.

**Test:** VPS sends TTS, watch plays it.

### 9.5 Voice Pipeline
- Wake word → stream → VPS → TTS → playback.
- End-to-end latency under 2 seconds.

**Test:** Say "Hey Watch, what time is it", hear response.

---

## Phase 10: Sync Foundation

**Goal:** Watch syncs logs and settings with VPS. Alarms sync. Checkpoints work.

### 10.1 Sync Protocol
- Define message format (CBOR or custom).
- Define checkpoint semantics.
- Define ACK handling.

**Test:** Send dummy record, receive ACK.

### 10.2 Log Sync
- Read unsynced logs from TicKV.
- Push to VPS.
- Mark as synced on ACK.

**Test:** Generate logs, sync, verify VPS has them.

### 10.3 Settings Sync
- Pull settings from VPS.
- Apply to watch.
- Push local changes.

**Test:** Change setting on VPS, watch receives it.

### 10.4 Alarm Sync
- Pull alarms from VPS.
- Store in TicKV.
- Push local alarms.

**Test:** Create alarm on VPS, watch vibrates on time.

### 10.5 Conflict Resolution
- Define precedence: VPS wins or watch wins.
- Handle duplicate records.

**Test:** Modify same setting on both, verify resolution.

---

## Phase 11: AI Integration

**Goal:** VPS node handles LLM, tools, TTS. Watch is thin client.

### 11.1 VPS Node Setup
- Rust service on VPS.
- WebSocket server.
- DeepSeek API integration.
- Gemini TTS integration.

**Test:** Send text to VPS, receive TTS audio.

### 11.2 Tool Calling
- Define tool registry in Rust.
- Implement `save_note`, `get_notes`, `set_timer`.
- LLM calls tools, executes, responds.

**Test:** Say "save note milk", verify note stored.

### 11.3 Voice Command Pipeline
- Wake word → stream → VPS → LLM → tool → TTS → playback.
- End-to-end under 3 seconds.

**Test:** Say "set timer 5 minutes", watch vibrates after 5 minutes.

### 11.4 Offline Fallback
- VPS unreachable → queue request or local command.
- User notified.

**Test:** Disconnect VPS, say command, watch handles gracefully.

---

## Phase 12: Polish & Optimization

**Goal:** Battery life maximized. UI smooth. Edge cases handled.

### 12.1 Power Audit
- Measure current in every state.
- Identify leaks.
- Fix leaks.

**Test:** Watch mode idle current under 50 µA.

### 12.2 UI Polish
- Smooth animations.
- No dropped frames.
- Theme transitions.

**Test:** 60 FPS in desk mode. No visual glitches.

### 12.3 Error Handling
- Brownout graceful shutdown.
- Flash corruption recovery.
- Network failure handling.

**Test:** Simulate brownout, verify no corruption.

### 12.4 Battery Life Retention Test
- Full charge.
- Mixed usage: 23 hours watch mode, 1 hour desk mode.
- Measure and compare against the Phase 7.6 baseline.

**Test:** No regression against the baseline, and the target goal is met if reachable.

---

## Phase 13: Stretch Goals

**Goal:** Optional features if time permits.

### 13.1 Custom Case
- 3D printed case for larger battery.
- Speaker removal.

### 13.2 OTA Updates
- `edge-ota` integration.
- A/B partition.
- Rollback on failure.

### 13.3 Additional Apps
- Stopwatch.
- Timer.
- Step counter.
- Heart rate (if sensor added).

### 13.4 Advanced Voice
- Local command recognition (10 commands).
- Multi-turn conversation.
- Context awareness.

---

## Testing Discipline

- Every phase has a testable deliverable.
- No phase advances until deliverable is verified.
- Hardware tests on real device.
- Host render and lifecycle tests on desktop; there is no simulator target.
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
- Firmware spine sixth because one binary, one boot chain, and the mode seams
  must exist before either mode is built.
- Watch mode seventh because it is the primary use case and drives power.
- Desk mode eighth because it unlocks everything else.
- Voice ninth because it needs desk mode.
- Sync tenth because it needs storage and voice.
- AI eleventh because it needs voice and sync.
- Polish last because it needs everything.
