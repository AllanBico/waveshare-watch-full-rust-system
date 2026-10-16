### 🏗️ Project Structure: Workspace, Not Monolith

**Decision: Cargo workspace with separate crates.**
A single crate will become a dependency hell once you mix drivers, power, storage, UI, and voice. Separate crates for `hal`, `ui`, `storage`, `power`, `voice`, and the rest enforce boundaries. Yes, it's more setup, but it prevents the "one change breaks everything" problem. The workspace builds one firmware binary; the crates are internal seams, not separate programs.

### 🔄 Mode Switching: Runtime, Not Compile-Time

**Decision: One binary, runtime switch via AXP2101 VBUS detection.**
Shipping two firmwares is a maintenance nightmare and breaks the "plug in → desk mode" UX. Detect VBUS at boot and on every wake. The switch is graceful: finish current operation, flush critical state to TicKV, then reinitialize subsystems for the new mode. Hard-switch only on USB yank, and even then, save alarm state before dropping to watch mode.

**Consequence: the compiler is no longer the safety net.** Because voice and sync are always linked, watch-mode purity moves to a runtime state machine. Watch boot never initializes voice/sync and `PowerManager` keeps their rails off. This is verified by tests, not by feature flags.

### 🧩 Abstraction Boundaries: Device-Level Traits, Not Registers

**Decision: HAL abstracts at the device level.**
Define `Display`, `Touch`, `IMU`, `Power`, `Storage`, and `Audio` as traits. Register-level abstraction is too granular and locks you into one chip. Device-level traits let you swap the CO5300 for a different panel later, or swap TicKV for something else, without touching app code. The cost is a thin wrapper layer, but it pays for itself the first time you debug a hardware issue.

### ⚡ Power Management: Central Manager, Enforced Sleep

**Decision: One `PowerManager` struct owns all rails, sleep decisions, and the render policy.**
No subsystem touches the AXP2101 directly. The manager exposes `request_awake(reason, budget)` and `enter_sleep()`. Every wake must justify itself with a budget (e.g., 50ms for motion verify, 200ms for UI). If the budget expires, force sleep. This is the only way to prevent "death by a thousand cuts" where a forgotten timer keeps the CPU awake. The mode-switch checklist is owned here too, so one place kills the mic rail, stops the wake task, and clamps the clock.

### 🎨 UI Architecture: One Custom Engine, Policy-Driven

**Decision: In-house retained engine, no mirui, no ECS.**
We build one small engine that serves both modes. Draw calls target a `Surface` trait: watch renders to a 1-bit buffer in internal SRAM, desk renders to a 16-bit buffer in PSRAM. Behavior differences come from a runtime `RenderPolicy` (color mode, animations, pixel budget). Widgets read the policy and never branch on `Mode`. This keeps the engine `no_std`, free of a third-party dependency we'd have to trust for power-sensitive rendering, and light enough that an ECS would be dead weight. Dirty-rect diffing gives watch mode the partial refresh it needs; the same engine gives desk mode 16-bit color and animation.

### 🗄️ Data & Storage: Custom Time-Series on TicKV

**Decision: TicKV as the KV engine, custom time-series layer on top.**
TicKV is pure Rust `no_std`, power-loss resilient, and has wear leveling built in. It's exactly what you need for a watch. Build your time-series on top by storing each log entry under a monotonic key. This avoids FlashDB's C dependency and gives you full control over the log format.

### 🔊 Voice & AI: Local Wake Word, VPS for Everything Else

**Decision: hand-rolled Rust wake word locally, VPS for commands and TTS.**
There is no mature pure-Rust equivalent of microWakeWord, and TFLite is C, so we build the wake word ourselves. The pipeline is all `no_std`, all Rust: I2S capture, `microfft` FFT, MFCC features, a small neural net with `const` weights, and a confidence threshold. A cheap VAD gates it so the model only runs on speech energy. Everything after the wake word streams to the VPS. The watch never runs command recognition locally. The VPS handles endpointing so the watch doesn't need a second model.

### 🔌 Connectivity: WebSocket, PCM to the VPS

**Decision: WebSocket over WiFi in desk mode. The watch sends PCM; the VPS does Opus.**
HTTP is too heavy for streaming; WebSocket gives bidirectional streaming with acceptable latency. There is no pure-Rust Opus encoder, so the watch does not run Opus. It captures PCM (or a simple Rust-side compression) and streams it. The VPS, which is free to use C, handles Opus encode/decode, TTS, and endpointing. The watch uses a small ring buffer for DMA playback, never storing the full audio clip.

### 🧪 Development Workflow: Simulator First, Hardware Second

**Decision: Desktop simulator target from day one.**
Compile the UI for desktop with an SDL backend before touching hardware. The custom engine's `Surface` trait already supports this. You'll catch layout bugs, theme issues, and animation problems without flashing. Hardware testing is for power, timing, and peripheral integration. This saves hours of flash cycles.

### 💡 MVP: Clock + Log + Sleep

**Decision: The smallest thing that proves the architecture works.**
A monochrome clock that wakes on motion, logs wake reasons to TicKV, and returns to deep sleep. No voice, no sync, no themes. If this works and the battery lasts, the architecture is sound. Everything else is incremental.

### 🧠 Philosophical: Build the Watch You Have

**Decision: Don't abstract for the watch you might build someday.**
Every abstraction has a cost. If you're not going to swap the display, don't abstract the display driver beyond a thin trait. If you're not going to use a different IMU, don't build an IMU abstraction layer. This is also why there's no ECS and no third-party UI engine: we're building the watch on the desk, not a framework. Refactor when the need actually arises.

### 🌐 Scope: Watch vs VPS

**Decision: pure Rust applies to the watch, not the server.**
The watch is the constrained device: it must stay pure Rust, `no_std`, and free of C. The VPS is a normal Rust service on a machine that has room for C crates, Opus, and ML. Pushing weight to the VPS is the whole point: the watch stays thin, the server stays capable.
