# Firmware Spine Gate — Phase 6.6

Status: code-side checks pass. Hardware rows are pending on-device execution.

Use this checklist before starting Phase 7. A failed row blocks the phase.

## Automated checks

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` | clean |
| Host tests | `./tools/test-host.sh` | pass |
| Firmware clippy | `. /home/jarvis/export-esp.sh && cargo clippy -p firmware` | clean |
| Firmware build | `. /home/jarvis/export-esp.sh && cargo build -p firmware` | pass; only the pre-existing RWX linker warning |
| Preview source absent | `test ! -e firmware/src/preview && test ! -e crates/apps/src/preview` | pass |
| Preview symbols absent | `strings target/xtensa-esp32s3-none-elf/debug/firmware \| grep -i preview` | no matches |
| No duplicate boot bodies | `grep -rn "fn watch_init\|fn desk_init" firmware/src` | no matches |

## Code-review gate

- One boot chain runs from `main`: PMIC → RTC → mode → storage/settings →
  PSRAM → display → mode peripherals.
- Allowed mode seams only:
  - `boot::Plan` for PSRAM, touch, and logging capabilities.
  - `session::run` for the awake watch/desk state machine.
  - `present::render` for mono versus gray presentation.
  - `sleep_cycle::enter` for the pre-sleep checklist and deep-sleep call.
  - Storage provisioning policy in `persist` and `log_boot`.
- No duplicated `watch_init` or `desk_init` bodies.
- Watch boot does not initialize the heap, PSRAM, touch, audio, radios, voice, or sync.
- The retired preview is absent from the current tree and the firmware binary.
- Watch boot writes no serial output; bring-up counters live in the retained ring.

## Boot matrix

| Case | Steps | Expected | Result |
|---|---|---|---|
| Watch cold boot | Flash, unplug USB, reset on battery | Serial: `cpu: 80 MHz`, `mode: Watch`, `psram: disabled`; panel shows Home; sleeps after the awake budget | pending |
| Desk cold boot | Flash, keep USB connected, reset | Serial: `mode: Desk`, `psram: 8388608 bytes`, `[boot] policy ...`; panel shows Home; stays awake | pending |
| Boot error visibility | Remove the panel or force a display init failure | Serial reports `[lcd] ...`; boot does not silently hang | pending |

## Live switch matrix

Run each switch while the device is awake.

| Case | Steps | Expected | Result |
|---|---|---|---|
| Watch → desk | During the watch awake window, plug USB | Serial: `[pm] switched Watch -> Desk` and `[boot] psram ...`; panel switches to the gray Home; stays awake | pending |
| Desk → watch | While in desk, unplug USB | Serial: `[pm] switched Desk -> Watch`; panel switches to the mono Home; sleeps after the awake budget | pending |
| Ten cycles | Plug/unplug ten times, staying within the awake window | No crash, no reset, Home follows the mode, state preserved | pending |
| Retained switch log | After the cycles, reconnect USB and reboot | Log sequence advances; retained ring flushed on the desk boot | pending |
| PSRAM survives | Watch → desk → watch → desk | Desk Home still renders after repeated switches | pending |

## Sleep and wake matrix

| Case | Steps | Expected | Result |
|---|---|---|---|
| Alarm wake | Set an alarm, let the device sleep, wait for it | Retained ring records Alarm; Home shows with the alarm | pending |
| Button wake | Press PWR while asleep | Retained ring records Button; Home shows; mode matches VBUS | pending |
| Motion wake | Move the device while asleep | Retained ring records Motion; Home shows; mode matches VBUS | pending |
| USB while asleep (fallback) | Sleep on battery, plug USB, then move or press | Device boots with `mode: Desk` and shows Home | pending |
| Awake budget | Leave watch awake without input or USB | Device returns to deep sleep after the policy budget | pending |

The AXP IRQ path is not implemented because `EXIO5`/`AXP_IRQ` is not mapped to
an ESP GPIO in the repository. The documented fallback above is the gate. Add
the IRQ to Ext1 only after the schematic or expander path is confirmed.

## Settings gate

| Check | Expected | Result |
|---|---|---|
| Default boot policy | Serial `[boot] policy`: `wake_on_motion: true`, `motion_threshold_mg: 80`, `awake_budget_ms: 7000`, `haptics: 60` | pending |
| Watch brightness | Panel is visible at the default 70% watch brightness | pending |
| Desk brightness | Panel is visible at the default 70% desk brightness | pending |
| Theme | Default theme renders in both modes | pending |
| Non-default values | Covered by `settings::policy` host tests until the Settings app exists (8.4) | host pass |

Settings that are intentionally not wired in this phase: `power_saving`,
`display.always_on`, and `display.screen_timeout_secs`.

## Known temporary items

- Both modes boot at 80 MHz; the desk may raise clocks in Phase 8.
- Desk audio is not initialized; it returns in Phase 8.
- PSRAM is initialized lazily on the first desk entry.
- VBUS insert while asleep relies on motion, button, or alarm wake.

## Sign-off

| Date | Firmware commit | Automated checks | Hardware checks | Notes |
|---|---|---|---|---|
| | | | | |
