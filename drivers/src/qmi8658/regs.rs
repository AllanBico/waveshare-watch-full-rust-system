//! QMI8658C registers, configuration values, and data layout.
//!
//! From the QMI8658C datasheet (Rev 0.6), Tables 20, 24, 26, 27, and 33.

/// Device identity register.
pub(super) const WHO_AM_I: u8 = 0x00;
/// Device revision register.
pub(super) const REVISION_ID: u8 = 0x01;
/// Serial interface and sensor enable.
pub(super) const CTRL1: u8 = 0x02;
/// Accelerometer settings.
pub(super) const CTRL2: u8 = 0x03;
/// Gyroscope settings.
pub(super) const CTRL3: u8 = 0x04;
/// Sensor data processing settings.
pub(super) const CTRL5: u8 = 0x06;
/// Sensor enable bits.
pub(super) const CTRL7: u8 = 0x08;
/// Host command register.
pub(super) const CTRL9: u8 = 0x0A;
/// Wake-on-Motion threshold and interrupt settings.
pub(super) const CAL1_L: u8 = 0x0B;
/// Wake-on-Motion interrupt and blanking settings.
pub(super) const CAL1_H: u8 = 0x0C;

/// Interrupt and command status.
///
/// Newer QMI8658 firmware reports CTRL9 completion on bit 7 here instead of
/// STATUS1 bit 0. Both are polled.
pub(super) const STATUSINT: u8 = 0x2D;
/// Miscellaneous status, including CmdDone and WoM.
pub(super) const STATUS1: u8 = 0x2F;
/// First acceleration output byte.
pub(super) const AX_L: u8 = 0x35;
/// First angular-rate output byte.
#[cfg(test)]
pub(super) const GX_L: u8 = 0x3B;

/// Expected [`WHO_AM_I`] value.
pub(super) const CHIP_ID: u8 = 0x05;

/// CTRL1: enable address auto-increment for burst reads.
pub(super) const CTRL1_AUTO_INCREMENT: u8 = 1 << 6;
/// CTRL1: SPI big-endian, the power-on default.
pub(super) const CTRL1_SPI_BE: u8 = 1 << 5;
/// CTRL1: route interrupt output to INT1.
///
/// The datasheet marks this bit reserved, but QST's reference drivers set it
/// to enable the INT1 pin. Without it, Wake-on-Motion sets STATUS1 but never
/// asserts the physical interrupt.
pub(super) const CTRL1_INT1_ENABLE: u8 = 1 << 3;
/// CTRL1 after init: auto-increment, default SPI order, INT1 enabled.
pub(super) const CTRL1_INIT: u8 = CTRL1_AUTO_INCREMENT | CTRL1_SPI_BE | CTRL1_INT1_ENABLE;

/// Accelerometer full scale: ±4 g.
pub(super) const ACCEL_FS_4G: u8 = 0b001 << 4;
/// Accelerometer ODR: 125 Hz normal mode.
pub(super) const ACCEL_ODR_125HZ: u8 = 0b0110;
/// Accelerometer ODR: 128 Hz low-power mode (gyro must be off).
pub(super) const ACCEL_ODR_LP_128HZ: u8 = 0b1100;

/// Gyroscope full scale: ±512 dps.
pub(super) const GYRO_FS_512DPS: u8 = 0b101 << 4;
/// Gyroscope ODR: 125 Hz.
pub(super) const GYRO_ODR_125HZ: u8 = 0b0110;

/// CTRL7: enable the accelerometer.
pub(super) const CTRL7_AEN: u8 = 1 << 0;
/// CTRL7: enable the gyroscope.
pub(super) const CTRL7_GEN: u8 = 1 << 1;

/// CTRL9: configure Wake-on-Motion.
pub(super) const CTRL9_WRITE_WOM_SETTING: u8 = 0x08;
/// CTRL9: no operation.
pub(super) const CTRL9_NOP: u8 = 0x00;

/// CAL1_H bits 7:6: INT1 with initial value 0.
pub(super) const CAL1_H_INT1_INITIAL_LOW: u8 = 0b00 << 6;
/// CAL1_H bits 5:0: interrupt blanking count mask.
pub(super) const CAL1_H_BLANKING_MASK: u8 = 0x3F;

/// STATUSINT bit 7: newer firmware's CTRL9 completion flag.
pub(super) const STATUSINT_CMD_DONE: u8 = 1 << 7;
/// STATUS1 bit 0: the datasheet's CTRL9 completion flag.
pub(super) const STATUS1_CMD_DONE: u8 = 1 << 0;
/// STATUS1 bit 2: a Wake-on-Motion event occurred.
pub(super) const STATUS1_WOM: u8 = 1 << 2;

/// Bytes in one accelerometer sample.
pub(super) const ACCEL_LEN: usize = 6;
/// Bytes in one gyroscope sample.
pub(super) const GYRO_LEN: usize = 6;
/// Bytes in the contiguous accelerometer-plus-gyroscope block.
pub(super) const MOTION_LEN: usize = ACCEL_LEN + GYRO_LEN;

/// Accel sensitivity at ±4 g.
pub(super) const ACCEL_4G_LSB_PER_G: u16 = 8_192;
/// Gyro sensitivity at ±512 dps.
pub(super) const GYRO_512_LSB_PER_DPS: u16 = 64;
