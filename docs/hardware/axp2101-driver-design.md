# AXP2101 Driver — Phase 1.1 Design

Status: agreed design, not yet implemented. Mechanical driver only; all power
policy lives in `power`.

## Scope
- Full mechanical register API: identity, status, voltages, rails, IRQ, ADC.
- Probe-only constructor. No rail writes on init.
- Blocking I2C on a shared bus. `no_std`. No allocation. No panics.

## Bus and handle
- One I2C bus (SDA=GPIO15, SCL=GPIO14, 400 kHz) shared by AXP2101, FT3168,
  QMI8658, PCF85063A, ES8311. See `board-pins.md`.
- Wrap the bus once in a blocking `Mutex`; each driver holds an
  `embedded_hal_bus` `MutexDevice` handle.
- `Axp2101<D: I2c>`; methods take `&mut self`.
- The I2C address is a constructor argument supplied by `board.rs`
  (`board::PMIC_I2C_ADDR`), never hardcoded inside the driver.

## Layering
- Private raw layer: `read_u8`, `write_u8`, `update_u8(mask, value)`,
  `read_seq(&mut [u8])`.
- Public typed layer built on the raw layer.

## Public API
Identity / probe
- `new(device, address) -> Result<Self, AxpError>` — reads chip ID (`0x03`),
  expects `0x4A`.
- `read_chip_id() -> Result<u8, AxpError>`

Status
- `read_status() -> Result<Status, AxpError>`

Voltages (two-tier)
- `read_battery_millivolts() -> Result<Millivolts, AxpError>` — wake path, one read.
- `read_wake_power_state() -> Result<WakePowerState, AxpError>` — wake path,
  `{ battery, vbus_present }`.
- `read_power_snapshot() -> Result<PowerSnapshot, AxpError>` — desk/UI; bursts
  `0x00..0x01` and `0x34..0x3B`, then `0xA4`. All-or-nothing on bus error.

Rails (mechanical)
- `set_rail_voltage(rail: Rail, mv: Millivolts) -> Result<(), AxpError>`
- `enable_rail(rail: Rail, on: bool) -> Result<(), AxpError>`
- `is_rail_enabled(rail: Rail) -> Result<bool, AxpError>`

Charger / ADC
- `set_charge_target(target: ChargeTarget) -> Result<(), AxpError>`
- `set_adc_channel(channel: AdcChannel, on: bool) -> Result<(), AxpError>`
- `set_battery_detection(on: bool) -> Result<(), AxpError>` — required before
  reading battery percent.

IRQ
- `set_irq_enabled(mask: IrqMask, on: bool) -> Result<(), AxpError>`
- `clear_irq(mask: IrqMask) -> Result<(), AxpError>`
- `pending_irq() -> Result<IrqMask, AxpError>`

Lifecycle
- `power_off() -> Result<(), AxpError>` (reg `0x10` bit 0)
- `reset_pmic() -> Result<(), AxpError>` (reg `0x10` bit 1)

Power levers (mechanical; `power` owns policy)
- `set_low_battery_warning(percent: Percent) -> Result<(), AxpError>` (reg `0x1A` high nibble, 5–20%)
- `set_low_battery_shutdown(percent: Percent) -> Result<(), AxpError>` (reg `0x1A` low nibble, 0–15%)
- `set_system_shutdown_voltage(mv: Millivolts) -> Result<(), AxpError>` (reg `0x24`, VSYS floor)
- `set_dcdc_mode(rail: Rail, mode: DcdcMode) -> Result<(), AxpError>` (reg `0x81`, `Auto` | `Pwm`)
- `set_internal_discharge(on: bool) -> Result<(), AxpError>` (reg `0x10` bit 5; bleeds unused outputs)
- `set_dvm_ramp(ramp: DvmRamp) -> Result<(), AxpError>` (reg `0x80` bit 5; slew for inrush control)
- `read_die_temperature() -> Result<u16, AxpError>` (regs `0x3C..0x3D`)
- `set_pwron_long_press(config: PwronLongPress) -> Result<(), AxpError>` (reg `0x22`)

## Types
- Newtypes: `Millivolts(u16)`, `Percent(u8)` (clamped 0..=100).
- `Status { vbus_good, vbus_in, battery_present, charging, charge_state, power_on }`
- `ChargeState { Tri, Pre, Cc, Cv, Done, Stop }` — from `0x01 & 0x07`.
- `PowerSnapshot { battery, vbus, system, percent, status }`
- `WakePowerState { battery, vbus_present }`
- `Rail` enum (`#[non_exhaustive]`): `DCDC1..DCDC4`, `ALDO1..ALDO4`,
  `BLDO1..BLDO2`, `CPUSLDO`. Only rails wired on this board.
- `AdcChannel { BatteryVoltage, TsPin, VbusVoltage, SystemVoltage, DieTemp }`
- `ChargeTarget { V4_0, V4_1, V4_2, V4_35, V4_4 }`
- `IrqMask(u32)` bitset with named consts (`VBUS_INSERT`, `VBUS_REMOVE`,
  `BAT_INSERT`, `PKEY_SHORT`, `CHARGE_DONE`, `LOW_BATTERY`, …).

## Error
`AxpError<E>`: `Bus(E)`, `BadChipId(u8)`, `OutOfRange`, `OffStep`,
`NotRepresentable`. Driver does not depend on `common`; `power` maps this to
`common::Error` (`HardwareFailure` / `PowerFail`). `defmt` behind a feature.

## Rail voltage encoding
Piecewise per datasheet ranges:
- DCDC1: 1.5–3.4 V, 100 mV.
- DCDC2: 0.5–1.2 V @10 mV, 1.22–1.54 V @20 mV.
- DCDC3: 0.5–1.2 V @10 mV, 1.22–1.54 V @20 mV, 1.6–3.4 V @100 mV.
- DCDC4: 0.5–1.2 V @10 mV, 1.22–1.84 V @20 mV.
- DCDC5: not wired.
- ALDO1–4 / BLDO1–2: 0.5–3.5 V, 100 mV.
- CPUSLDO: 0.5–1.4 V, 50 mV.
Const `RailSpec` tables; off-step or out-of-range returns an error. Round-trip
encode/decode is unit-tested.

## Module layout
```
drivers/src/board.rs          pin map + I2C addresses
drivers/src/axp2101/mod.rs    Axp2101<D>, public API
drivers/src/axp2101/regs.rs   register + bitfield constants
drivers/src/axp2101/rails.rs  Rail, RailSpec, encode/decode
drivers/src/axp2101/status.rs Status, ChargeState, PowerSnapshot, IrqMask
drivers/src/axp2101/error.rs  AxpError
drivers/src/axp2101/mock.rs   #[cfg(test)] MockI2c
```

## Testing
`MockI2c` implements `embedded_hal::i2c::I2c` against a `[u8; 256]` register
file, logs writes, and can inject NACKs. Covers: chip-ID probe, voltage decode,
burst atomicity, rail encode/decode + error paths, IRQ mask, error mapping.

## Out of scope for 1.1
- Bus recovery on a stuck SDA (belongs to the bus owner in `firmware`).
- Charge-current limits, watchdog config, gauge ROM writes.
- Any rail write on init; `power` applies the vendor sequence.

## Hardware notes baked in
- Battery NTC on TS; follow vendor default (TS measurement disabled) in 1.1.
- RTCLDO powers the RTC through sleep; `power` leaves it alone.
- PWROK → ESP32 `CHIP_PU`; enables a real power-off path.
- AXP IRQ net (`EXIO5`) may route through an expander; 1.4 polls VBUS, so fine.
