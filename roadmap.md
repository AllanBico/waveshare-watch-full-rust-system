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

### 0.3 Core Types
- Define `Mode`, `Error`, `Event`, `Timestamp`, `LogEntry` in `common`.
- No hardware dependencies.

**Test:** Unit tests for type construction and conversion.

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

**Test:** Set the time, read it back, and confirm it advances. Timestamps are available for logs.

---

## Phase 2: Storage Foundation

**Goal:** TicKV works, custom time-series layer works, data survives reboot.

### 2.1 TicKV Integration
- Flash partition for TicKV.
- `Storage` struct wraps TicKV.
- Basic `set`, `get`, `delete`.

**Test:** Write key, reboot, read key. Value persists.

### 2.2 Time-Series Layer
- `append_log(entry)` assigns monotonic key.
- `read_logs_since(checkpoint)` returns entries.
- `prune_logs(before_timestamp)` removes old entries.

**Test:** Append 100 entries, read back, verify order and content.

### 2.3 Logging Crate
- RAM ring buffer for runtime events.
- `log!` macro compiles to nothing when `logging` feature is off.
- Flush to `storage` on demand.

**Test:** Log 50 events, flush, read back from TicKV.

### 2.4 Battery Voltage Logging
- Log battery voltage on every wake.
- Store in time-series.

**Test:** Wake 10 times, read 10 voltage entries.

---

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

**Test:** Draw a clock. Count lit pixels. Under 5%.

### 3.4 Display Rail Control
- Cut display rail via AXP2101 before sleep.
- Restore on wake.

**Test:** Sleep, measure current. Display rail off saves measurable power.

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

## Phase 5: UI Foundation

**Goal:** The custom engine renders, the theme/policy system works, one app renders.

### 5.1 UI Engine Skeleton
- Define the `Surface` trait and a software renderer backend.
- Define the `RenderPolicy` (color mode, animations, pixel budget).
- Render a static label to the display.

**Test:** "Hello" appears on screen.

### 5.2 Theme System
- Define theme tokens: colors, spacing, fonts.
- Two themes: watch (monochrome), desk (16-bit).
- Runtime switch driven by the render policy.

**Test:** Switch theme, UI updates.

### 5.3 App Trait
- `App` trait: `render()`, `handle_event()`, `on_enter()`, `on_exit()`.
- App registry with names.

**Test:** Register two apps, switch between them.

### 5.4 Clock App
- Monochrome clock.
- Updates once per minute.
- Dirty rect only.

**Test:** Clock displays correct time. Updates without full redraw.

### 5.5 Simulator
- Desktop target for UI.
- Same code renders to SDL window.

**Test:** Run clock app on desktop.

---

## Phase 6: Watch Mode Integration

**Goal:** Full watch mode works end-to-end. Battery lasts measurable time.

### 6.1 Watch Mode Entry
- Boot into watch mode.
- Initialize minimum peripherals.
- Enter deep sleep.

**Test:** Watch boots, shows clock, sleeps.

### 6.2 Wake Cycle
- Wake on motion, button, or timer.
- Verify wake reason.
- Show clock if appropriate.
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

**Test:** Plug USB, UI switches to color theme.

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
