# Power Strategy — Watch Mode

Status: agreed direction for Phases 1.2+ (watch-mode power). The AXP2101 driver
(`axp2101-driver-design.md`) supplies the primitives; `power` owns all policy.

## Wake sources (silicon constraint)
esp-hal 1.0.0 marks GPIO0–21 as RTC pins (deep-sleep wake capable). Therefore:

| Source | Pin | Deep-sleep wake |
|---|---|---|
| BOOT button | GPIO0 | yes |
| PWR button | GPIO10 | yes |
| IMU motion | GPIO21 | yes |
| Touch | GPIO38 | **no** |
| PCF85063A RTC INT | GPIO39 | **no** |

- Wake set: **motion + button + internal RTC timer**. Touch cannot wake the SoC.
- Alarms are scheduled on the ESP32-S3 internal RTC timer (compute delta before
  sleep); the external RTC is wall-clock persistence only.
- Confirm whether the AXP IRQ (`EXIO5` net) reaches a wake pin. If not, USB
  insert cannot wake a sleeping watch — it is noticed on the next wake.

## Display strategy
- Watch mode sleeps with the panel rail cut. Clock shows on motion/tap/button.
- IMU stays powered in wake-on-motion (it is a wake source).
- Always-on display (AOD) is a possible later mode, not the default.

## Power state table (single source of truth)
One table maps state to actuators; `PowerManager` applies transitions.

| State | CPU | Panel | Radios | Voice/sync | IMU |
|---|---|---|---|---|---|
| DeepSleep | off | rail cut | off | off | wake-on-motion |
| Aod (optional) | 80 MHz | dim mono | off | off | wake-on-motion |
| ActiveWatch | 80 MHz | on mono | off | off | on |
| Desk | 240 MHz | color | on | on | on |

The power estimator reads this same table so display and behavior cannot drift.

## Rail audit (pre-sleep and post-wake)
- Not "all rails off": each managed rail must match the state table. RTCLDO, the
  IMU rail, and the pull-up/AXP supply stay on in sleep.
- "Off" is not automatically "no leak": GPIOs attached to unpowered peripherals
  must be high-Z/input before sleep.
- Enable AXP internal discharge on rails we cut to remove residual voltage.
- Static audit: read `is_rail_enabled` back vs the table; mismatch -> log + safe
  state. Testable against the mock PMIC.
- Dynamic audit: measured sleep current at/below baseline is the real proof.

## Rail sequencing (per transition)
- Power-down is the reverse of power-up. One documented order, owned by `power`.
- Panel: power-down = `SLPIN` -> stop QSPI -> cut rail. Power-up = rail -> reset
  -> init. Never drive QSPI into an unpowered panel.
- Use DVM ramp for inrush control; bring up the main rail deliberately.
- Kill the SoC's own load last so a heavy rail never drops under a running SoC.
- Delays are named constants; each step is logged as an event.

## Measurement and accounting
- AXP2101 has no current ADC: on-device current cannot be measured.
- Bench-measure per-state current once (multimeter/INA219), store as constants.
- On-device: accumulate time-in-state x per-state current to get mAh, and
  cross-check against AXP percent/voltage drop; apply a calibration factor.
- Never claim precision the hardware cannot provide.

## Storage and budget interactions
- Battery voltage is logged every wake, but to a RAM ring buffer; flash flush is
  batched (desk mode or every N wakes). Never write to flash every wake.
- Every wake has a budget; on expiry, force sleep and log the reason.

## Target
- Roadmap 11.1's sub-50 µA idle is likely unreachable with AXP + IMU + RTC + LDO
  quiescent current. Measure a baseline, set a realistic goal, and never regress.
