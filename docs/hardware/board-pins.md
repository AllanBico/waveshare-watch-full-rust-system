# Board Pin Map — Waveshare ESP32-S3-Touch-AMOLED-2.06

Reference source: `pin_config.h` from the Waveshare Arduino examples.
Canonical home when implemented: `drivers/src/board.rs`. This file is the raw
reference; no pin or address may be hardcoded anywhere else.

```rust
// === QSPI Display (CO5300) ===
pub const LCD_SDIO0: u8 = 4;
pub const LCD_SDIO1: u8 = 5;
pub const LCD_SDIO2: u8 = 6;
pub const LCD_SDIO3: u8 = 7;
pub const LCD_SCLK: u8 = 11;
pub const LCD_CS: u8 = 12;
pub const LCD_RESET: u8 = 8;
pub const LCD_WIDTH: u16 = 410;
pub const LCD_HEIGHT: u16 = 502;
pub const LCD_COL_OFFSET: u16 = 22;
pub const LCD_ROW_OFFSET: u16 = 0;

// === I2C Bus ===
pub const I2C_SDA: u8 = 15;
pub const I2C_SCL: u8 = 14;
pub const I2C_FREQ_HZ: u32 = 400_000;

// === Touch (FT3168) ===
pub const TP_INT: u8 = 38;
pub const TP_RESET: u8 = 9;
pub const TP_I2C_ADDR: u8 = 0x38;

// === Power (AXP2101) ===
pub const PMIC_I2C_ADDR: u8 = 0x34;

// === IMU (QMI8658) ===
pub const IMU_I2C_ADDR: u8 = 0x6B;

// === RTC (PCF85063A) ===
pub const RTC_I2C_ADDR: u8 = 0x51;

// === SD Card ===
pub const SD_CLK: u8 = 2;
pub const SD_CMD: u8 = 1;
pub const SD_DATA: u8 = 3;
pub const SD_CS: u8 = 17;

// === Display TE (Tearing Effect sync) ===
pub const LCD_TE: u8 = 13;

// === Audio I2S ===
pub const I2S_MCLK: u8 = 16;
pub const I2S_SCLK: u8 = 41;   // BCLK
pub const I2S_LRCK: u8 = 45;   // WS
pub const I2S_DSDIN: u8 = 40;  // DAC data in (speaker)
pub const I2S_ASDOUT: u8 = 42; // ADC data out (microphone)
pub const PA_CTRL: u8 = 46;    // Power amplifier enable

// === IMU Interrupt ===
pub const IMU_INT: u8 = 21;

// === RTC Interrupt ===
pub const RTC_INT: u8 = 39;

// === Buttons ===
pub const BOOT_BUTTON: u8 = 0;
pub const PWR_BUTTON: u8 = 10;
```

## Notes to verify

- **QSPI pins (4–7, 11, 12, 8)** are confirmed against the vendor demo
  `05_LVGL_AXP2101_ADC_Data.ino`: `Arduino_ESP32QSPI(CS, SCLK, SDIO0..3)` and
  `Arduino_CO5300(bus, LCD_RESET, 0, width, height, 22, 0, 0, 0)`.
  So `LCD_RESET` (GPIO8) is real and `LCD_COL_OFFSET = 22` is confirmed.
- **No vibration motor pin exists** in the pin map or the vendor demos. Roadmap
  6.3 (alarm vibrate) assumes one. Replace vibration with a display/sound alert
  unless a motor is confirmed.
- **AMOLED brightness** is not a PWM backlight pin. The CO5300 exposes brightness
  via a panel command (e.g. `0x51`) over QSPI. No backlight GPIO.
- **SD card** is not used by the roadmap's storage plan (TicKV on internal
  flash). Treat SD as optional assets/export only.
- **I2C is shared** by AXP2101, FT3168, QMI8658, PCF85063A, and the ES8311 codec.
  All at 400 kHz on SDA=15, SCL=14.
