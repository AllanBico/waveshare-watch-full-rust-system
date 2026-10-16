### 🏗️ Workspace Layout

```
wv2/
├── Cargo.toml              # workspace root (virtual manifest)
├── .cargo/config.toml      # target, runner, linker config
├── rust-toolchain.toml     # pin toolchain version
├── build.rs                # top-level build orchestration
├── contracts/              # wire contract + golden vectors (shared with VPS)
├── crates/
│   ├── hal/                # hardware abstraction traits
│   ├── power/              # power manager, rails, sleep, render policy
│   ├── storage/            # TicKV wrapper + time-series layer
│   ├── ui/                 # custom engine: theme, widgets, renderer, animation
│   ├── apps/               # screens: clock, settings, logs, voice
│   ├── voice/              # wake word, audio capture, VPS client
│   ├── sync/               # VPS sync protocol, checkpoints
│   ├── logging/            # ring buffer, wake reason tracking
│   └── common/             # shared types, errors, events
├── firmware/               # ONE crate, ONE binary; main branches to watch_init/desk_init
├── drivers/                # chip drivers (CO5300, FT3168, QMI8658, AXP2101, PCF85063A, ES8311); board.rs pin map
├── vps/                    # companion Rust server (separate binary, separate rules)
├── assets/                 # fonts, icons, wake word model weights
├── tools/                  # host-side utilities (log parser, sim)
└── simulator/              # desktop UI simulator
```

### 🔌 Hardware Target

**Board: Waveshare ESP32-S3-Touch-AMOLED-2.06 (confirmed on silicon).**

| Component | Part | Bus | Notes |
|-----------|------|-----|-------|
| SoC | ESP32-S3R8, rev v0.2, dual Xtensa LX7, 40 MHz xtal | — | 8 MB octal PSRAM, 32 MB flash |
| Display | CO5300 AMOLED 410×502 | QSPI 80 MHz | rounded edge, RGB565 |
| PMIC | AXP2101 | I2C 400 kHz | rails, charging, battery |
| Touch | FT3168 | I2C + INT | INT on GPIO38 |
| IMU | QMI8658 | I2C | accel + gyro + temp |
| RTC | PCF85063A | I2C | BCD time/date |
| Audio | ES8311 + PA amp | I2C + I2S | amp enable on GPIO46 |
| Tearing | TE signal | GPIO13 | VSync for flush |
| Button | boot button | GPIO0 | also a wake source |

- **No user LED.** Bring-up is verified over USB serial, not by blinking.
- Serial enumerates as `/dev/ttyACM0` (native USB Serial/JTAG).
- All pins live in `drivers/src/board.rs`. No pin or address is hardcoded elsewhere.
- Chip identity (model, flash size, MAC) is read at runtime, never hardcoded.

### 🔀 One Binary, Two Modes

`firmware/` builds a single binary. `main` reads VBUS from the AXP2101 and calls either `watch_init` or `desk_init`. `watch_init` brings up only the minimum peripherals and enters the sleep loop; it never starts voice, sync, or radios. `desk_init` brings up the full set. Mode is runtime state owned by `power`, never a compile-time feature.

Until Phase 0.2, `firmware` is a `no_std` library exposing `watch_init` and `desk_init`. Linking a bare-metal binary needs the Xtensa GCC linker and `esp-hal`, both introduced with hardware bring-up.

### 🎯 Purpose of Each Crate

**`common`**
Shared types that every other crate needs: `Mode` enum, `Error` types, `Event` enum, `Timestamp`, `LogEntry` struct. No hardware, no allocation beyond what's necessary. This is the vocabulary of the system. Every other crate depends on it, and it depends on nothing. It is named `common` rather than `core` because a crate named `core` shadows the `core` prelude and breaks every dependent crate.

**`hal`**
Trait definitions only. `Display`, `Touch`, `IMU`, `Power`, `Storage`, `Audio`. No implementations. This is the contract that drivers fulfill and apps consume. Keeps app code hardware-agnostic.

**`drivers`**
Concrete implementations of the HAL traits for your specific chips: CO5300 display, FT3168 touch, QMI8658 IMU, AXP2101 PMIC, PCF85063A RTC, ES8311 codec. `board.rs` holds the single pin map. Each driver is a separate module. Drivers know about registers and I2C addresses. They do not know about modes or power budgets.

**`power`**
The `PowerManager`. Owns the AXP2101 handle and the runtime `RenderPolicy`. Exposes `request_awake(reason, budget_ms)` and `enter_sleep()`. Knows which rails to cut, which peripherals to disable, and when to force sleep. Owns the single mode-switch checklist. Every wake path and every mode transition goes through here. This is the single source of truth for power state and render behavior.

**`storage`**
Wraps TicKV. Provides two layers: a raw KV API (`set`, `get`, `delete`) and a time-series API (`append_log`, `read_logs_since`, `prune_logs`). Owns the flash partition and the retention policy. Never exposes TicKV types directly—if you swap TicKV later, only this crate changes.

**`logging`**
A RAM ring buffer for runtime events (wake reasons, mode switches, errors). Flushes to `storage` only in desk mode or on significant events. Provides the `log!` macro that compiles to nothing when the `logging` feature is off. This is what feeds your power analysis.

**`ui`**
The custom engine: theme tokens, widget tree, layout, renderer backend, dirty-rect diffing, animation. Depends on `hal` for the display trait and `common` for events. Does not know about storage, power, or voice. Receives a `RenderPolicy` and a `Surface` and renders the current app state. Backend-agnostic: hardware display on target, SDL on desktop.

**`apps`**
Each app is a module implementing a common `App` trait: `render()`, `handle_event()`, `on_enter()`, `on_exit()`. Apps depend on `ui` for widgets, `common` for events, and optionally `storage` or `voice` if they need data. The clock app needs nothing but `ui`. The settings app needs `storage`. The voice app needs `voice`. Apps never touch hardware directly, and never branch on `Mode`.

**`voice`**
Wake word, audio capture, PCM streaming, VPS client, TTS playback. The wake word is a hand-rolled `no_std` pipeline (I2S, `microfft`, MFCC, small NN with `const` weights) gated by VAD. Contains no Opus; the VPS does that. Only activated in desk mode. Depends on `hal` for the audio trait and `common` for events.

**`sync`**
VPS sync protocol, checkpoint tracking, conflict resolution. Depends on `storage` to read unsynced records and the network layer to transmit. Only active in desk mode.

**`firmware`**
The single entry point and binary. Reads VBUS, dispatches to `watch_init` or `desk_init`, and runs the mode state machine. It owns initialization; every other crate is a component it wires together.

**`vps`**
The companion Rust service. WebSocket server, LLM agent, tool registry, Opus encode/decode, TTS, endpointing, and sync endpoint. Governed by the server guide, not this one. Free to use C crates.

### 🔗 Dependency Direction

```
common ← everything
hal ← drivers, ui, voice
drivers ← power, firmware
power ← firmware
storage ← logging, sync, apps
ui ← apps, firmware
apps ← firmware
voice ← firmware
sync ← firmware
```

**Rule: dependencies point inward, never outward.** `ui` does not depend on `apps`. `apps` does not depend on `firmware`. `drivers` do not depend on `power`. This keeps the graph acyclic and every crate testable in isolation.

### 🚩 Feature Flags (Defined on `firmware`)

A virtual workspace manifest cannot declare `[features]`, so the flags live on the `firmware` package and propagate to members from there.

```toml
[features]
default = []
logging = []      # RAM ring buffer, flush to TicKV
trace = []        # verbose structured tracing for development
simulator = []    # desktop UI target; never compiles hardware drivers
```

**Note:** There are no `watch` or `desk` features. Mode is runtime state, not a build variant. Features are reserved for genuine build variants only.

### 🧪 What This Gives You

- **Testability**: Each crate has a clear boundary. `ui` can be tested on desktop. `storage` can be tested with a mock flash. `power` can be tested with a mock PMIC.
- **Replaceability**: Swap TicKV for something else by changing only `storage`. Swap the display driver by changing only `drivers`. Swap the UI engine by changing only `ui`.
- **Runtime safety**: Watch-mode purity is enforced by the mode state machine and tested, not by the compiler. Every mode switch runs one checklist owned by `power`.
- **Clear ownership**: Every piece of state has exactly one owner. `PowerManager` owns rails and render policy. `storage` owns flash. `ui` owns the framebuffer.

### 💡 One Rule to Enforce

**No crate outside `drivers` touches a register. No crate outside `power` touches a rail. No crate outside `storage` touches flash.** If you enforce only these three rules, the rest of the architecture will hold together.
