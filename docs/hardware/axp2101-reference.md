# AXP2101 Reference — Raw Notes

Inspiration source: `05_LVGL_AXP2101_ADC_Data.ino` (Waveshare example).
Status: raw reference only. The production driver lives in `drivers/` and must
be verified against the AXP2101 datasheet before shipping. Do not copy the
challenged parts below without review (see "Open questions").

## Registers used

```rust
const AXP2101_ADDR: u8 = 0x34;

const REG_STATUS1: u8 = 0x00;
const REG_STATUS2: u8 = 0x01;
const REG_IC_TYPE: u8 = 0x03;

const REG_VBAT_H: u8 = 0x34;
const REG_VBAT_L: u8 = 0x35;
const REG_TS_H: u8 = 0x36;
const REG_TS_L: u8 = 0x37;
const REG_VBUS_H: u8 = 0x38;
const REG_VBUS_L: u8 = 0x39;
const REG_VSYS_H: u8 = 0x3A;
const REG_VSYS_L: u8 = 0x3B;

const REG_DC_ONOFF: u8 = 0x80;     // DC output on/off + DVM control
const REG_DC_VOL0: u8 = 0x82;      // DCDC1 voltage setting
const REG_LDO_ONOFF0: u8 = 0x90;   // ALDO1-4 on/off control
const REG_LDO_VOL0: u8 = 0x92;     // ALDO1 voltage setting

const REG_ADC_ENABLE: u8 = 0x30;
const REG_IRQ_ENABLE0: u8 = 0x40;
const REG_IRQ_ENABLE1: u8 = 0x41;
const REG_IRQ_ENABLE2: u8 = 0x42;
const REG_IRQ_STATUS0: u8 = 0x48;
const REG_IRQ_STATUS1: u8 = 0x49;
const REG_IRQ_STATUS2: u8 = 0x4A;

const REG_BAT_PERCENT: u8 = 0xA4;
const REG_CHG_STATUS: u8 = 0x01;
```

## Reference init sequence

```rust
// DC1 = 3300mV: reg 0x82, value = (3300 - 1500) / 100 = 18
write_reg(REG_DC_VOL0, 18);
// Enable DC1: reg 0x80, bit 0
write_reg(REG_DC_ONOFF, read_reg(REG_DC_ONOFF) | 0x01);

// ALDO1 = 3300mV: reg 0x92, value = (3300 - 500) / 100 = 28
write_reg(REG_LDO_VOL0, 28);
// Enable ALDO1: reg 0x90, bit 0
write_reg(REG_LDO_ONOFF0, read_reg(REG_LDO_ONOFF0) | 0x01);

// Disable all IRQs, clear pending by writing 1s
write_reg(REG_IRQ_ENABLE0, 0x00);
write_reg(REG_IRQ_ENABLE1, 0x00);
write_reg(REG_IRQ_ENABLE2, 0x00);
write_reg(REG_IRQ_STATUS0, 0xFF);
write_reg(REG_IRQ_STATUS1, 0xFF);
write_reg(REG_IRQ_STATUS2, 0xFF);

// Enable ADC: 0b00011101 = VBAT + VBUS + VSYS + die temp
write_reg(REG_ADC_ENABLE, 0b00011101);
```

- Battery voltage: `((high << 8) | low) & 0x3FFF`, comment claims 1.1 mV/LSB.
- VBUS / VSYS voltage: same 14-bit mask.
- Battery percent: `read(REG_BAT_PERCENT)`.
- Charging: `(read(0x01) >> 5) & 0x07`, values 1..=3 treated as charging.
- VBUS present: `read(REG_STATUS1) & 0x20`.
- ADC trim: clear die-temp bit → `0b00001101` (VBAT + VBUS + VSYS).

## Verified against XPowersLib + vendor demo (2026-10-17)

Sources: `lewisxhe/XPowersLib` (`XPowersAXP2101.hpp`, `AXP2101Constants.h`,
`XPowersCommon.hpp`) and the vendor repo
`waveshareteam/ESP32-S3-Touch-AMOLED-2.06`
(`examples/arduino/examples/05_LVGL_AXP2101_ADC_Data`).

1. **ADC scaling is 1 mV/LSB, not 1.1 mV/LSB.** Confirmed by the AXP2101
   datasheet (Table 6-7): VBAT/VBUS/VSYS = 1 mV per LSB (0..8.191 V), TS =
   0.5 mV, die temp = 0.1 mV. The vendor demo also prints the values with a
   literal `"mV"` suffix. The pasted "1.1 mV/LSB" is the AXP192 figure and is
   wrong for AXP2101. Datasheet note: read the high bits first, then the low
   byte ("TWSI must read the high 6 bits firstly and then the low 8 bits").
2. **Register widths differ per channel.** XPowersLib combines:
   - VBAT  = `((reg34 & 0x1F) << 8) | reg35` (5 high bits)
   - VBUS  = `((reg38 & 0x3F) << 8) | reg39` (6 high bits)
   - VSYS  = `((reg3A & 0x3F) << 8) | reg3B` (6 high bits)
   - TS/die-temp = 14-bit `((reg&0x3F)<<8)|reg`.
   The pasted code masks everything with `0x3FFF`; battery must use `0x1FFF`.
3. **Charging decode is wrong in the paste.** `isCharging()` = STATUS2 (`0x01`)
   bits[7:5] **== 0x01 exactly**. 0x02 is *discharging*, 0x00 is standby. The
   paste treats 1..=3 as charging.
   `getChargerStatus()` is a *different* field: `0x01 & 0x07`, with values
   0 tri / 1 pre / 2 CC / 3 CV / 4 done / 5 stop.
4. **VBUS present.** "VBUS good" = STATUS1 (`0x00`) bit 5. "VBUS in" per
   XPowersLib = STATUS2 bit 3 == 0 *and* VBUS good. Battery present = STATUS1 bit 3.
5. **Battery percent needs detection enabled, not a gauge bit.** The vendor demo
   calls `enableBattDetection()` (reg `0x68` bit 0) and only then reads
   `REG_BAT_PERCENT` (`0xA4`). There is no gauge-enable bit in the ADC register
   `0x30` (bit 7 there is TS low-frequency sample; bit 5 is general ADC).
   The earlier "fuel-gauge bit" hypothesis was wrong.
6. **ADC channel enable bits (`0x30`) are correct as written.**
   bit0 VBAT, bit1 TS, bit2 VBUS, bit3 VSYS, bit4 die temp, bit5 general,
   bit7 TS low-freq. `0b00011101` and the trim `0b00001101` are valid.
7. **Chip ID = `0x4A`** at reg `0x03`. Validate during init; XPowersLib's whole
   `initImpl()` is just: if chip id == 0x4A, disable TS-pin measure, else fail.
8. **Rails are not enabled by the vendor example.** The demo never enables DCDC1
   or ALDO1; it relies on whatever default/bootloader state the PMIC powers up in.
   The pasted `init()` rail writes come from somewhere else and must be validated
   on hardware before adoption (and belong in `power`, not `drivers`).
9. **Vendor PMIC setup sequence:** `disableIRQ(ALL_IRQ)` →
   `setChargeTargetVoltage(3)` → `clearIrqStatus()` → `enableIRQ(PKEY_SHORT)` →
   enable ADC channels. Adopt the IRQ/charge part in `power`, not the driver.
10. **Duplicate constant.** `REG_CHG_STATUS` (0x01) == `REG_STATUS2` (0x01).
    Keep one name.

## Decisions (Phase 1.1)

- **Bus sharing:** one I2C bus behind `embedded-hal-bus`; each driver owns a
  cheap `Device` handle (chosen over raw-bus passing and async-from-day-one).
- **Rail policy:** `power` owns rails; the AXP2101 driver exposes mechanical
  register primitives only. No high-level `init()` in the driver.
- **Scope:** full register API now — voltages, rails, IRQ, ADC.
- **Voltage API:** two-tier — `read_battery_millivolts()` and
  `read_wake_power_state()` for the wake path, `read_power_snapshot()` for
  desk/UI. Snapshot is all-or-nothing on bus error.
- **Units:** newtypes `Millivolts(u16)` and `Percent(u8)`.
- **Rail coverage:** only the wired families — DCDC1–4, ALDO1–4, BLDO1–2,
  CPUSLDO — behind a `#[non_exhaustive]` enum; append others later.
- **Bus:** blocking I2C shared via `embedded-hal-bus` `MutexDevice`.
- **Verify first:** datasheet/vendor checks done above before writing code.

## Board rail map (from the AXP2101 schematic)

Wired AXP2101 outputs on this board:

| Rail | Net | Voltage | Notes |
|------|-----|---------|-------|
| DCDC1 | VCC3V3 | 3.3 V | main rail |
| DCDC2 | — | 0.9 V | |
| DCDC3 | — | 1.2 V | |
| DCDC4 | — | 1.8 V | |
| DCDC5 | NC | — | not used |
| ALDO1 | VL1 | 3.3 V | |
| ALDO2 | VL2 | 3.3 V | |
| ALDO3 | VCC3V3 | 3.3 V | |
| ALDO4 | VL3 | 1.8 V | |
| BLDO1 | VL | 1.2 V | |
| BLDO2 | VL | 2.8 V | |
| CPUSLDO | VCL | 1.2 V | |
| RTCLDO | VCC-RTC | — | RTC keep-alive rail |
| VBAT1 | CHG BAT | — | main battery (connector J1) |
| VBAT2 | CHG RTC | — | RTC backup (VBACKUP) |

Other wiring observed on the sheet:

- Battery NTC 10K thermistor on the TS pin. The vendor disables TS measurement
  (AXP ignores TS). Follow that default in 1.1; revisit for charge safety.
- AXP IRQ (pin 38) shares a net labelled `EXIO5` / `AXP IRQ`. Confirm whether it
  reaches a direct ESP32 GPIO or an I/O expander. Phase 1.4 polls VBUS, so not
  blocking for power bring-up.
- PWROK (pin 29) ties to the ESP32 `CHIP_PU`. The AXP can gate ESP32 power/reset —
  relevant to `power_off()` and brownout handling (Phases 1.2, 11.3).

**Consequence for the rail API:** the board uses DCDC, ALDO, **BLDO**, and
**CPUSLDO**, plus the RTC rail. The "families we use" set must include those, not
just DCDC/ALDO.

## Still open

- Which peripheral consumes each net (VL1/VL2/VL3/VCL) — needs the top-level
  sheet, not the PMIC sheet. Not needed for the 1.1 driver.
- `setChargeTargetVoltage` encoding (datasheet: VBATREG = 4.0/4.1/4.2/4.35/4.4 V)
  and whether it matters before Phase 2.
- Whether the board has a vibration motor (no pin found; see `board-pins.md`).
