//! PCF85063A register addresses and bit fields.
//!
//! From the NXP PCF85063A datasheet (Rev. 7). See `docs/datasets/PCF85063A.pdf`.

/// Control_1: control and status register.
pub(super) const CTRL1: u8 = 0x00;
/// Control_2: interrupts and CLKOUT.
pub(super) const CTRL2: u8 = 0x01;
/// Seconds (bit 7 = OS, oscillator stop).
pub(super) const SECONDS: u8 = 0x04;

/// Control_1: STOP the RTC divider chain.
pub(super) const CTRL1_STOP: u8 = 1 << 5;
/// Control_1: 12-hour mode (clear for 24-hour).
pub(super) const CTRL1_12_HOUR: u8 = 1 << 1;
/// Control_1 value that triggers a software reset.
pub(super) const CTRL1_SOFT_RESET: u8 = 0x58;

/// Control_2: CLKOUT frequency field mask.
pub(super) const CTRL2_COF_MASK: u8 = 0x07;
/// Control_2: CLKOUT off (output low).
pub(super) const CTRL2_COF_OFF: u8 = 0x07;

/// Seconds bit 7: oscillator stop flag.
pub(super) const SECONDS_OS: u8 = 1 << 7;

/// Seconds field mask.
pub(super) const SECONDS_MASK: u8 = 0x7F;
/// Minutes field mask.
pub(super) const MINUTES_MASK: u8 = 0x7F;
/// Hours field mask (24-hour mode).
pub(super) const HOURS_MASK: u8 = 0x3F;
/// Days field mask.
pub(super) const DAYS_MASK: u8 = 0x3F;
/// Months field mask.
pub(super) const MONTHS_MASK: u8 = 0x1F;

/// Length of the seconds..years time block.
pub(super) const TIME_BLOCK_LEN: usize = 7;
/// Index of seconds within the time block.
pub(super) const BLOCK_SECONDS: usize = 0;
/// Index of minutes within the time block.
pub(super) const BLOCK_MINUTES: usize = 1;
/// Index of hours within the time block.
pub(super) const BLOCK_HOURS: usize = 2;
/// Index of days within the time block.
pub(super) const BLOCK_DAYS: usize = 3;
/// Index of months within the time block.
pub(super) const BLOCK_MONTHS: usize = 5;
/// Index of years within the time block.
pub(super) const BLOCK_YEARS: usize = 6;
