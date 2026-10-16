//! Board pin map and I2C addresses.
//!
//! The single source of truth for pins and bus addresses for the Waveshare
//! ESP32-S3-Touch-AMOLED-2.06. No pin or address is hardcoded anywhere else.
//! Values are confirmed against the vendor `pin_config.h` and schematic V1.0.

/// QSPI display data line 0.
pub const LCD_SDIO0: u8 = 4;
/// QSPI display data line 1.
pub const LCD_SDIO1: u8 = 5;
/// QSPI display data line 2.
pub const LCD_SDIO2: u8 = 6;
/// QSPI display data line 3.
pub const LCD_SDIO3: u8 = 7;
/// QSPI display clock.
pub const LCD_SCLK: u8 = 11;
/// QSPI display chip select.
pub const LCD_CS: u8 = 12;
/// Display reset (active low).
pub const LCD_RESET: u8 = 8;
/// Display width in pixels.
pub const LCD_WIDTH: u16 = 410;
/// Display height in pixels.
pub const LCD_HEIGHT: u16 = 502;
/// Display column start offset.
pub const LCD_COL_OFFSET: u16 = 22;
/// Display row start offset.
pub const LCD_ROW_OFFSET: u16 = 0;
/// Display tearing-effect sync pin.
pub const LCD_TE: u8 = 13;

/// Shared I2C data pin.
pub const I2C_SDA: u8 = 15;
/// Shared I2C clock pin.
pub const I2C_SCL: u8 = 14;
/// Shared I2C bus frequency.
pub const I2C_FREQ_HZ: u32 = 400_000;

/// Touch interrupt pin.
pub const TP_INT: u8 = 38;
/// Touch reset pin.
pub const TP_RESET: u8 = 9;
/// FT3168 touch controller I2C address.
pub const TP_I2C_ADDR: u8 = 0x38;

/// AXP2101 PMIC I2C address.
pub const PMIC_I2C_ADDR: u8 = 0x34;

/// QMI8658 IMU I2C address.
pub const IMU_I2C_ADDR: u8 = 0x6B;
/// IMU interrupt pin.
pub const IMU_INT: u8 = 21;

/// PCF85063A RTC I2C address.
pub const RTC_I2C_ADDR: u8 = 0x51;
/// RTC interrupt pin (not deep-sleep wake capable).
pub const RTC_INT: u8 = 39;

/// SD card clock.
pub const SD_CLK: u8 = 2;
/// SD card command (MOSI).
pub const SD_CMD: u8 = 1;
/// SD card data (MISO).
pub const SD_DATA: u8 = 3;
/// SD card chip select.
pub const SD_CS: u8 = 17;

/// I2S master clock.
pub const I2S_MCLK: u8 = 16;
/// I2S bit clock.
pub const I2S_SCLK: u8 = 41;
/// I2S word select.
pub const I2S_LRCK: u8 = 45;
/// I2S DAC data (speaker).
pub const I2S_DSDIN: u8 = 40;
/// I2S ADC data (microphone).
pub const I2S_ASDOUT: u8 = 42;
/// Audio power-amplifier enable.
pub const PA_CTRL: u8 = 46;

/// Boot button (also a deep-sleep wake source).
pub const BOOT_BUTTON: u8 = 0;
/// Power button (also a deep-sleep wake source).
pub const PWR_BUTTON: u8 = 10;
