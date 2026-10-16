//! CO5300 register and QSPI instruction constants.
//!
//! Instructions are the quad-SPI opcodes the panel expects; commands are the
//! MIPI-DCS-style driver-IC registers.

/// QSPI instruction: command / register access.
pub const OP_WRITE_COMMAND: u8 = 0x02;
/// QSPI instruction: four-line pixel write with a 24-bit address.
pub const OP_WRITE_PIXELS: u8 = 0x32;
/// RAMWR-continuous command encoded as the pixel-write address.
pub const PIXEL_WRITE_ADDRESS: u32 = 0x0000_3C00;

/// Sleep out.
pub const CMD_SLPOUT: u8 = 0x11;
/// Sleep in.
pub const CMD_SLPIN: u8 = 0x10;
/// Inversion off.
pub const CMD_INVOFF: u8 = 0x20;
/// Display off.
pub const CMD_DISPOFF: u8 = 0x28;
/// Display on.
pub const CMD_DISPON: u8 = 0x29;
/// Column address set.
pub const CMD_CASET: u8 = 0x2A;
/// Page address set.
pub const CMD_PASET: u8 = 0x2B;
/// Memory write start.
pub const CMD_RAMWR: u8 = 0x2C;
/// Memory data access control.
pub const CMD_MADCTL: u8 = 0x36;
/// Interface pixel format.
pub const CMD_PIXFMT: u8 = 0x3A;
/// Brightness in normal mode.
pub const CMD_BRIGHTNESS: u8 = 0x51;
/// Write CTRL display 1.
pub const CMD_WCTRLD1: u8 = 0x53;
/// Write CE (contrast enhancement).
pub const CMD_WCE: u8 = 0x58;
/// Brightness in high-brightness mode.
pub const CMD_BRIGHTNESS_HBM: u8 = 0x63;
/// SPI mode control (enables SPI WRAM writes).
pub const CMD_SPIMODECTL: u8 = 0xC4;
/// Write command-mode page selector.
pub const CMD_WRITE_CMD_PAGE: u8 = 0xFE;

/// RGB565 interface pixel format value.
pub const PIXFMT_RGB565: u8 = 0x55;
/// Vendor register page holding the SPI mode control.
pub const CMD_PAGE_VENDOR: u8 = 0x00;
/// Enable the SPI WRAM write path.
pub const SPIMODECTL_WRAM_ENABLE: u8 = 0x80;
/// Default display brightness.
pub const BRIGHTNESS_DEFAULT: u8 = 0xD0;
/// Maximum high-brightness value.
pub const BRIGHTNESS_HBM: u8 = 0xFF;
/// CTRL display 1 default.
pub const WCTRLD1_DEFAULT: u8 = 0x20;
/// Contrast enhancement off.
pub const WCE_OFF: u8 = 0x00;
/// No axis flip, RGB color order.
pub const MADCTL_RGB: u8 = 0x00;
