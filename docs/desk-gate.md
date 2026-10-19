# Desk Gate — Phase 8.6

Status: host-side checks pass. Hardware rows are pending on-device execution.

The gate covers the scoped desk slice: Home, drawer with Settings and Logs,
blank quick settings, Settings, and Logs. Animations, calibration, battery
stats, voice, sync, and AI are intentionally deferred.

## Automated checks

| Check | Command | Result |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` | pass |
| Host tests | `./tools/test-host.sh` | pass |
| Firmware clippy | `. /home/jarvis/export-esp.sh && cargo clippy -p firmware` | pass |
| Firmware build | `. /home/jarvis/export-esp.sh && cargo build -p firmware` | pass; only the pre-existing RWX linker warning |
| Preview sources absent | `test ! -e firmware/src/preview && test ! -e crates/apps/src/preview` | pass |
| Preview symbols absent | `strings target/xtensa-esp32s3-none-elf/debug/firmware \| grep -i preview` | no matches |
| Preview tag absent | `git tag -l \| grep -i preview` | no output |

## Desk UI matrix

| Case | Steps | Expected | Result |
|---|---|---|---|
| Desk cold boot | Flash, keep USB connected, reset | Serial: `mode: Desk`, `psram: 8388608 bytes`; panel shows the gray Home; device stays awake | pending |
| App drawer | Swipe up from the bottom edge | Two columns with Settings and Logs; no placeholder tiles | pending |
| Quick settings | Swipe down from the top edge | Blank sheet; tap closes it | pending |
| Settings | Open Settings from the drawer | Time/date/brightness/sleep rows show `Live` or `Next sleep`; Theme appears only in desk | pending |
| Theme | Tap Theme | One full redraw; high-contrast desk theme applies immediately | pending |
| Logs | Open Logs from the drawer | Category list opens; a category shows entries with paging | pending |
| Brightness | Change brightness in Settings | Panel brightness changes immediately | pending |
| Persistence | Change theme and brightness, reboot on USB | Both settings are restored | pending |

## Mode and power matrix

| Case | Steps | Expected | Result |
|---|---|---|---|
| Watch → desk | While an app or overlay is open, plug USB | Serial: `[pm] switched Watch -> Desk`; panel lands on desk Home; active app is closed | pending |
| Desk → watch | While an app or overlay is open, unplug USB | Serial: `[pm] switched Desk -> Watch`; panel lands on watch Home; active app is closed | pending |
| Desk stays awake | Leave desk idle on USB longer than the watch awake budget | Device never sleeps; clock keeps advancing | pending |
| Desk rails | Inspect or log the rail plan in desk | Desk-only rails `Dcdc2`, `Bldo2`, and `Cpusldo` are on; audit passes | pending |
| Watch rails | Unplug and inspect or log the rail plan | Desk-only rails are off; audit passes | pending |
| Watch sleep | After returning to watch, leave it idle | Device sleeps after the watch idle budget | pending |
| Ten cycles | Plug/unplug ten times, opening apps between switches | No crash, no reset, Home follows the mode, persistent settings survive | pending |

## Known deferred work

- Animations, transitions, calibration, dialogs, and toast polish.
- Battery stats screen.
- Voice, sync, and AI.
- Runtime re-clocking from 80 MHz to 240 MHz.
