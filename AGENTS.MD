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
- Watch mode: 80 MHz max, radios off, deep sleep default, monochrome only, no animations, no background jobs.
- Desk mode: 240 MHz allowed, radios on, animations on, voice active, sync active.
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
- On switch to watch: kill the mic/model rail, stop the wake task, flip the render policy to mono/no-anim, clamp CPU to 80 MHz.
- Battery figures are goals, not gates. Measure current in every state and never regress.
- Logging only in desk mode. Never in watch mode.

## Memory Rules
- `no_std` does not mean no heap. An allocator (`esp-alloc`) is available in the one binary.
- Heap is a policy: allowed in desk mode for the UI, never used in the wake/sleep/render hot path.
- `heapless` for queues and buffers on hot paths.
- PSRAM for framebuffers and large assets only.
- Internal SRAM for hot paths.
- Wake stub uses RTC memory only.
- No dynamic allocation in the wake path.

## UI Rules
- One in-house engine serves both modes. No mirui, no ECS.
- Backend-agnostic: draw calls target a `Surface` trait, so watch (1-bit SRAM) and desk (16-bit PSRAM) share code.
- Behavior is driven by a runtime `RenderPolicy`: color mode, animations, pixel budget.
- Widgets never branch on `Mode`. They read the policy.
- Dirty rects only. Full redraw only on mode switch.
- Theme tokens, not hardcoded values.
- Watch mode: monochrome, no animations, under 5% lit pixels.
- Desk mode: 16-bit color, animations allowed.
- Desk-only assets must be gated by the policy at load time, not just at draw time.
- No layout changes without testing in the simulator first.

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
- `core` depends on nothing.
- `hal` defines traits only.
- `drivers` implement traits.
- `power` owns rails and the render policy.
- `storage` owns flash.
- `ui` owns the framebuffer.
- `apps` own their state.
- `firmware` is one crate, one binary. `main` branches to `watch_init` or `desk_init` after the VBUS read.

## Testing Rules
- Simulator first. Hardware second.
- Unit tests for logic. Integration tests for drivers.
- Power measurements on hardware only.
- Every feature must be testable in isolation.
- No feature ships without a way to verify it.
- Every mode-switch transition has a test.

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
- The simulator must never compile hardware drivers.
- Feature flags must be documented in the workspace `Cargo.toml`.

## Naming Rules
- Crates: lowercase, single word where possible.
- Types: PascalCase.
- Functions: snake_case.
- Constants: SCREAMING_SNAKE_CASE.
- Modes: `Mode::Watch`, `Mode::Desk`.
- Events: `Event::WakeReason`, `Event::ModeSwitch`.
- Errors: `Error::PowerFail`, `Error::StorageFull`.

## Documentation Rules
- Every public trait must have a doc comment.
- Every module must have a module-level doc comment.
- Every unsafe block must have a safety comment.
- Every hardware assumption must be documented.
- Every power trade-off must be documented.

## Forbidden Patterns
- No `std` in any watch code path.
- No `println!` in production.
- No `panic!` in watch mode.
- No `loop {}` without sleep or await.
- No `delay_ms()` in main loop.
- No global mutable state.
- No circular dependencies.
- No hardcoded pin numbers outside `drivers`.
- No hardcoded I2C addresses outside `drivers`.
- No magic numbers. Use named constants.

## Review Checklist
- Does this compile as `no_std` for the ESP32-S3 target?
- Does watch boot leave voice, sync, and radios uninitialized?
- Does this respect the power budget?
- Does this touch a rail without going through `power`?
- Does this touch flash without going through `storage`?
- Does this allocate in a hot path?
- Does this block in an async task?
- Does this assume hardware state?
- Does this have a test?
- Does this have a doc comment?

## Escalation Rules
- If unsure about power impact, ask.
- If unsure about mode behavior, ask.
- If unsure about hardware assumption, ask.
- If unsure about dependency direction, ask.
- Never guess. Never assume. Never skip the checklist.
