//! FT3168 registers and touch-data layout.
//!
//! The provided FT3168 datasheet covers modes and timing but not the register
//! map. Register addresses and power-mode values below follow the
//! register-compatible FocalTech FT5x06/FT6x06 family and vendor board
//! examples. Verify on hardware before treating them as final.

/// Touch status and finger count (low nibble).
pub(super) const REG_TOUCH_DATA: u8 = 0x02;
/// Chip identity, diagnostic only.
pub(super) const REG_CHIP_ID: u8 = 0xA0;
/// Device power mode.
pub(super) const REG_POWER_MODE: u8 = 0xA5;

/// Actively scans and reports coordinates.
pub(super) const POWER_ACTIVE: u8 = 0x00;
/// Stops scanning and turns off the analog circuits.
pub(super) const POWER_SLEEP: u8 = 0x03;

/// Finger-count field in [`REG_TOUCH_DATA`].
pub(super) const TOUCH_COUNT_MASK: u8 = 0x0F;
/// Coordinate high-nibble field.
pub(super) const COORD_HIGH_MASK: u8 = 0x0F;
/// Touch-ID field in the Y-high byte.
pub(super) const TOUCH_ID_SHIFT: u8 = 4;

/// Bytes per touch point: XH, XL, YH, YL, weight, misc.
pub(super) const POINT_STRIDE: usize = 6;
/// Status byte offset in a touch-data burst.
pub(super) const BLOCK_STATUS: usize = 0;
/// First touch-point offset in a touch-data burst.
pub(super) const BLOCK_POINT_BASE: usize = 1;
/// X-high offset within one point.
pub(super) const BLOCK_X_HIGH: usize = 0;
/// X-low offset within one point.
pub(super) const BLOCK_X_LOW: usize = 1;
/// Y-high offset within one point.
pub(super) const BLOCK_Y_HIGH: usize = 2;
/// Y-low offset within one point.
pub(super) const BLOCK_Y_LOW: usize = 3;

/// Length of the status-plus-points burst.
pub(super) const TOUCH_BUFFER_LEN: usize =
    BLOCK_POINT_BASE + common::MAX_TOUCH_POINTS * POINT_STRIDE;
