# Power Estimator Reference — Raw Notes

Inspiration: a lightweight mA-budget estimator with runtime tracking. Status:
raw reference, no code adopted yet. Intended home is the `power` crate (it owns
the render policy and rails), as pure host-testable math.

## Shape (from inspiration)

- `DisplayState { Off, Aod, Dim, Bright }`
- `WifiMode { Off, PowerSave, Active }`
- `PowerStats { display, wifi, ble_on, imu_on, audio_on, sd_on, cpu_mhz,
  brightness, battery_mv, battery_pct, charging }`
- `base_ma()` scales with CPU clock (240→55, 160→38, 80→22, 40→14).
- `display_ma()` scales with brightness; AOD ~8 mA, Dim ~25 mA.
- `wifi_ma()` PowerSave ~20, Active ~90.
- Per-peripheral flat costs: BLE 15, IMU 2, audio 25, SD 30.
- `total_ma()`, `full_runtime_hours(capacity)`, `estimated_hours(capacity)`.

## Notes to verify / adapt to wv2

- Numbers are rough typicals, not telemetry. Label them as estimates in the UI.
- Pull the mA figures from the wv2 board measurements, not the inspiration.
- Replace magic mA numbers with named constants.
- Reuse `common`/`power` enums instead of new duplicate state enums where possible
  (`DisplayState` overlaps the `RenderPolicy`; `Mode` already exists).
- The inspiration includes BLE, but wv2 connectivity is "BLE for presence, WiFi
  for payload"; confirm BLE is actually used before budgeting it.
- No vibration motor may exist on this board (see `board-pins.md`); do not budget
  power for it until confirmed.
- This is a Phase 1.2+ / Phase 12 concern, not part of the 1.1 driver.
