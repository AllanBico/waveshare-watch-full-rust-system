## Project Identity
- Pure Rust `no_std` firmware for an ESP32-S3 watch, plus a companion VPS server.
- One firmware binary. Two runtime modes: watch (ultra low power) and desk (USB powered, full features).
- Custom UI engine (in-house), custom storage layer, custom power management.
- Watch firmware is pure Rust with no C, C++, or CMake dependencies.
- The VPS is a separate Rust service and may use C or C++ crates freely. It is not a watch deliverable.

## Scope
- This guide governs the watch firmware.
- The VPS is governed by the server guide under the server project, not this file.
- The single source of truth for the wire contract is `contracts/`. A contract change updates both sides and adds golden vectors.
- Rules marked "watch only" do not apply to the VPS.

## Absolute Rules
- Watch firmware: never introduce C, C++, or CMake dependencies.
- Never `unwrap()` or `expect()` in production paths. Tests may unwrap.
- Never busy-wait. Use interrupts, DMA, or async.
- Never touch a register outside `drivers`.
- Never touch a power rail outside `power`.
- Never touch flash outside `storage`.
- Never allocate in interrupt handlers.
- Never block in async tasks.
- Never assume hardware state. Read it.
- Never dynamic-allocate in the wake, sleep, or render hot path.

## Mode Rules
- One binary. Mode is chosen at runtime from AXP2101 VBUS detection only.
- Both modes are `no_std`. The compiler cannot separate them; the runtime state machine does.
- Watch mode: 80 MHz max, radios off, deep sleep default, 1-bit monochrome only, no animations, no background jobs.
- Desk mode: 240 MHz allowed, radios on, 8-bit grayscale monochrome, animations on, voice active, sync active.
- Watch boot must never initialize voice, sync, or radios. Their tasks are never spawned, and `power` keeps their rails off.
- Desk boot initializes the full set of subsystems.
- Mode switch must be graceful: flush state, finish the operation, then switch.
- A single transition checklist runs on every switch. It is owned by `power`.
- Desk mode must never assume it will stay in desk mode.
- Watch-mode purity is enforced by tests: assert no voice/sync task exists and the mic rail is off.

## Power Rules
- Every wake must have a reason and a time budget.
- Budget expires = force sleep. No exceptions.
- Sleep is the default state. Awake is the exception.
- Pre-sleep checklist must run before every sleep.
- `PowerManager` is the single owner of all power rails and of the render policy.
- On switch to watch: kill the mic/model rail, stop the wake task, flip the render policy to 1-bit/no-anim, clamp CPU to 80 MHz.
- Battery figures are goals, not gates. Measure current in every state and never regress.
- Logging only in desk mode. Never in watch mode.

## Memory Rules
- `no_std` does not mean no heap. An allocator (`esp-alloc`) is available in the one binary.
- Heap is a policy: allowed in desk mode for app data, never in the UI build, layout, paint, or flush path, and never in watch mode.
- The UI scene arena is fixed-capacity and reused. No per-frame allocation in either mode.
- `heapless` for queues and buffers on hot paths.
- Desk framebuffer (8-bit grayscale) in PSRAM. Watch framebuffer (1-bit) in internal SRAM.
- Internal SRAM for hot paths.
- Wake stub uses RTC memory only.
- No dynamic allocation in the wake path.

## UI Rules
- One in-house engine serves both modes. No mirui, no ECS, no retained widget tree with state.
- Both modes are monochrome on true black. Watch renders 1-bit; desk renders 8-bit grayscale with antialiasing.
- Draw calls target a `Surface` trait. `MonoSurface` (1-bit SRAM) and `GraySurface` (8-bit PSRAM) share one luminance painter.
- Behavior is driven by a runtime `RenderPolicy`: pixel mode, animations, pixel budget.
- Widgets never branch on `Mode`. They read the policy and the theme.
- Widgets are pure functions of props. Application state lives in apps, never in widgets.
- Widgets never touch hardware and never allocate.
- Build and layout first, then paint. Layout is deterministic and integer-only.
- Widget tree depth is capped at 32.
- The scene arena is fixed-capacity. No dynamic allocation in build, layout, paint, or flush.
- Stable widget IDs plus a previous-frame fingerprint snapshot drive damage automatically.
- Dirty rects only. Merge and coalesce before paint and cap the rect count. No full-screen dirty rect except boot, mode switch, or theme switch.
- Single framebuffer in both modes. The CO5300 has its own GRAM and TE; add an MCU double buffer only after measured tearing.
- Paint only inside damage rects. Never paint or flush what is not visible.
- No full framebuffer push except boot, mode switch, or theme switch. Use TE-synced partial writes.
- Watch: no animations, repaint only on state change, under 5% lit pixels.
- Desk: 60 fps cap, damage-based animation, 80 MHz QSPI target.
- Theme tokens only. No hardcoded tones, spacing, radii, borders, or motion values.
- Themes are compiled-in structs, never trait objects or globals. Runtime switch without reboot. A theme switch dirties all visible widgets.
- Fonts and icons are compiled in. No runtime font, icon, SVG, or theme parsing or loading.
- Text rendering lives only in `ui`. Text measurement is cached. Clock and data digits are tabular.
- Assets use 4-bit coverage masks. Watch thresholds to 1-bit; desk blends. Watch icon output is 1-bit, with no animated icons.
- Report asset sizes at build time; no asset over 64 KB without approval.
- No shadows, blur, gradients, or filters in v1.
- No layout changes without host golden-frame tests in both watch and desk.

## Display Rules
- All panel writes go through `hal::Display`; only the display adapter and the `ui` flush path touch the panel.
- Use TE to synchronize partial flushes. Never tear a frame.
- Flush only damaged regions. Mono expands to RGB565 in chunks; desk converts Gray8 through a luma LUT.
- Keep the desk framebuffer in PSRAM and the watch framebuffer in internal SRAM.
- Never drive QSPI into an unpowered panel. Sleep the panel before cutting its enable.
- After panel power loss, re-run full init and re-flush the retained frame.
- Target 80 MHz QSPI in desk. Keep watch at the lowest rate that meets the wake budget.

## Storage Rules
- TicKV as KV engine. Custom time-series on top.
- Batch writes. Never write on every wake.
- No writes below battery threshold.
- Log schema must be versioned.
- Retention policy must be explicit.
- Sync is one-way push with checkpoint.

## Voice Rules
- Voice is desk mode only. Watch boot never starts it.
- Wake word runs locally. It is the only thing that runs continuously on-device.
- Wake word is a hand-rolled `no_std` pipeline: I2S capture, `microfft` FFT, MFCC, a small NN with `const` weights, threshold. No allocator in this path, no TFLite, no `tract`.
- VAD gates the wake model so it only runs on speech energy.
- Everything after the trigger streams to the VPS. The watch does not run commands locally.
- The VPS does endpointing and Opus encode/decode. The watch sends PCM (or a simple Rust-side compression). No Opus on the watch.
- Mic and codec rails off in watch mode.
- Ring buffer for audio. Never buffer the full clip.
- VPS unreachable = queue or fail gracefully.

## Connectivity Rules
- WiFi and BLE are burst resources. Never left running.
- BLE for presence. WiFi for payload.
- WebSocket over WiFi in desk mode.
- Reconnection uses exponential backoff.
- Message framing must be versioned.
- TLS cost must be justified.

## Code Structure Rules
- Dependencies point inward. Never outward.
- `common` depends on nothing. (Named `common`, not `core`, since a crate named `core` shadows the `core` prelude.)
- `hal` defines traits only: `Display`, `Touch`, `IMU`, `Power`, `Storage`, `Audio`, `Time`.
- `drivers` implement traits.
- `power` owns rails and the render policy.
- `storage` owns flash.
- `ui` owns tokens, surfaces, layout, widgets, renderer, damage, and the framebuffer.
- `shell` owns the app trait, navigation, overlays, focus routing, dialogs, toasts, and app lifecycle. `shell` depends on `ui`, never on concrete apps.
- `apps` own their state and depend on `shell` and `ui`. Apps never implement global navigation.
- `firmware` is one crate, one binary. `main` branches to `watch_init` or `desk_init` after the VBUS read, and wires apps and services into the shell.

## Testing Rules
- Host tests first. Hardware second.
- Unit tests for logic. Integration tests for drivers.
- Power measurements on hardware only.
- Every feature must be testable in isolation.
- No feature ships without a way to verify it.
- Every mode-switch transition has a test.
- UI changes require golden-frame tests for both watch and desk.
- Damage-rect behavior is tested for move, insert, remove, and theme switch.
- Watch UI must pass the 5% lit-pixel budget.

## Logging Rules
- Logging is a feature flag. Off by default in watch mode.
- Log only state transitions, not continuous values.
- RAM ring buffer. Flush to TicKV only on significant events.
- Wake reasons logged always.
- Mode switches logged always.
- Errors logged always.
- Battery voltage logged on every wake.

## Error Handling Rules
- Every error must be handled or propagated.
- No silent failures.
- Panics wake the watchdog in desk mode.
- Watchdog disabled in watch mode.
- Brownout triggers graceful shutdown.
- Flash corruption triggers recovery from backup.

## Feature Flag Rules
- There are no `watch` or `desk` feature flags. Mode is runtime, not compile-time.
- Only genuine variants are features: `simulator`, `logging`, `trace`.
- The simulator is a host tool and test target, not a watch or desk mode.
- The simulator must never compile hardware drivers.
- Feature flags must be documented in the workspace `Cargo.toml`.

## File Size
- Max 300 lines per source file. If a source file grows past 300, split it.
- Design docs and the protocol document are exempt from the line limit.
- Split by responsibility, not by line count.
- One file, one job.

## Function Design
- One function, one purpose.
- If a function needs a comment to explain what it does, rename it.
- If a function needs a comment to explain why, add the comment.
- Max 40 lines per function.
- Max 5 parameters per function. If more, use a struct.
- Return `Result`, never panic in library code.
- Prefer pure functions when possible.
- No side effects unless the function's name says so.

## Naming Rules
- Crates: lowercase, single word where possible.
- Types: PascalCase.
- Functions: verb_noun, snake_case.
- Constants: SCREAMING_SNAKE_CASE.
- Booleans: `is_`, `has_`, `can_` prefix.
- Avoid abbreviations unless universally known.
- Modes: `Mode::Watch`, `Mode::Desk`.
- Events: `Event::WakeReason`, `Event::ModeSwitch`.
- Errors: `Error::PowerFail`, `Error::StorageFull`.

## Documentation Rules
- Comment the why, not the what.
- Do not comment obvious code.
- Do comment tricky logic, workarounds, and trade-offs.
- Keep comments short. One line if possible.
- Delete stale comments immediately.
- Mark temporary or placeholder code with a `TODO(phase-N):` comment naming the phase that removes or completes it. Remove the comment when the work is done.
- Every public trait must have a doc comment.
- Every public item must have a doc comment.
- Every module must have a module-level doc comment.
- Every unsafe block must have a safety comment.
- Every hardware assumption must be documented.
- Every power trade-off must be documented.

## No Hardcoding
- No magic numbers. Use named constants.
- No hardcoded paths, URLs, keys, or timeouts. Use the compile-time config struct.
- No hardcoded pin numbers or I2C addresses outside `drivers`. The single pin map lives in `drivers/src/board.rs`.
- Read chip identity (model, revision, flash size, PSRAM size, MAC) at runtime. Never hardcode what the silicon reports.
- If it might change, make it configurable. Config is a typed struct with safe, documented defaults.
- Secrets never live on the watch. If it is a secret, it belongs on the VPS.

## Forbidden Patterns
- No `std` in any watch code path.
- No `println!` in production.
- No `panic!` in watch mode.
- No `loop {}` without sleep or await.
- No `delay_ms()` in main loop.
- No global mutable state.
- No circular dependencies.
- No commented-out code.
- No debug prints left behind.

## Review Checklist
- Source file under 300 lines. Function under 40 lines.
- Does this compile as `no_std` for the ESP32-S3 target?
- Does watch boot leave voice, sync, and radios uninitialized?
- Does this respect the power budget?
- Does this touch a rail without going through `power`?
- Does this touch flash without going through `storage`?
- Does this allocate in a hot path?
- Does this block in an async task?
- Does this assume hardware state?
- No hardcoded values.
- No unsafe without justification.
- Errors handled. No silent failures.
- Comments explain why. Public items documented.
- Tests pass. Clippy clean. Rustfmt clean.

## Context Hygiene
- Never read screenshots, PDFs, or other binary assets inline. Convert to text or crop to the region needed. Never attach a full datasheet.
- Extract datasheets once with `pdftotext -layout` into `/tmp`, then search with targeted `grep` or `sed -n 'A,Bp'`.
- Do not dump whole files or directories. Search first with `grep` or `glob`, then read only the relevant line range.
- Keep command output small: `cargo check -q`, pipe through `tail` or `head`, and never paste full build logs.
- Cap tool output at roughly 4 KB. Summarize longer output and cite line numbers instead.
- One session per phase. Start a fresh session or compact at phase boundaries.
- Do not re-read `AGENTS.MD`, `GUIDE.MD`, or `roadmap.md` in full. Grep the section you need.
- Never search `target/`, `/tmp`, or `~/.cargo/registry` broadly. Scope every search to the relevant source or docs directory.

## Commit Rules
- Do not commit to git. Suggest commit messages only.
- A suggested message says what and why.
- One logical concern per suggested commit.
- No commented-out code.
- No debug prints left behind.
- No secrets in the suggested commit.

## Escalation Rules
- If unsure about power impact, ask.
- If unsure about mode behavior, ask.
- If unsure about hardware assumption, ask.
- If unsure about dependency direction, ask.
- Never guess. Never assume. Never skip the checklist.

fyi use . /home/jarvis/export-esp.sh to  dor esp
