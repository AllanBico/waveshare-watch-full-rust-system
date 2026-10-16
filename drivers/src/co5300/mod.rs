//! CO5300 AMOLED display driver.
//!
//! Quad-SPI AMOLED controller, 410x502 on the Waveshare ESP32-S3-Touch-AMOLED
//! 2.06. The driver owns the command set, reset sequence, address window, and
//! region writes. A [`Bus`] supplies the transport, and an output pin plus a
//! delay supply the hardware timing, so the whole driver is host-testable.
//!
//! Transport framing (see the ChipOne datasheet): register access uses the
//! `0x02` instruction on one line with a 24-bit address `{8'h00, command,
//! 8'h00}`; pixel data uses the `0x32` instruction on one line for instruction
//! and address, then four-line data.
//!
//! The controller's CASET/RASET require even start coordinates and even
//! extents, so windows are aligned outward (see [`Geometry::align_region`]).

use common::{Region, Rgb565};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use hal::Display;

mod bus;
mod error;
mod regs;

#[cfg(test)]
mod tests;

pub use bus::Bus;
pub use error::DisplayError;
pub use regs::{OP_WRITE_COMMAND, OP_WRITE_PIXELS, PIXEL_WRITE_ADDRESS};

use regs::*;

/// Reset assert pulse, in milliseconds.
const RESET_PULSE_MS: u32 = 10;
/// Reset hold and post-reset settle, in milliseconds.
const RESET_HOLD_MS: u32 = 200;
/// Sleep-out settle, in milliseconds.
const SLPOUT_MS: u32 = 120;
/// Sleep-in settle, in milliseconds.
const SLPIN_MS: u32 = 120;
/// Post-init settle, in milliseconds.
const INIT_SETTLE_MS: u32 = 10;

/// Panel geometry and its offset within the controller's address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Geometry {
    /// Logical width in pixels.
    pub width: u16,
    /// Logical height in pixels.
    pub height: u16,
    /// Column address offset.
    pub column_offset: u16,
    /// Row address offset.
    pub row_offset: u16,
}

impl Geometry {
    /// Expands `region` so its panel window meets the controller's alignment.
    ///
    /// CASET/RASET require both the start coordinate and the extent to be even,
    /// so an odd edge is grown outward by one pixel. The caller must supply
    /// pixels for the returned region, not the original.
    pub fn align_region(&self, region: Region) -> Region {
        let x0 = region.x + self.column_offset;
        let x1 = x0 + region.width.saturating_sub(1);
        let y0 = region.y + self.row_offset;
        let y1 = y0 + region.height.saturating_sub(1);
        let ax0 = x0 & !1;
        let ax1 = x1 | 1;
        let ay0 = y0 & !1;
        let ay1 = y1 | 1;
        Region::new(
            ax0.saturating_sub(self.column_offset),
            ay0.saturating_sub(self.row_offset),
            ax1 - ax0 + 1,
            ay1 - ay0 + 1,
        )
    }

    /// Whether a region already satisfies the panel's alignment.
    pub fn is_aligned(&self, region: Region) -> bool {
        self.align_region(region) == region
    }
}

/// A CO5300 AMOLED panel.
pub struct Co5300<B, R, D> {
    bus: B,
    reset: R,
    delay: D,
    geometry: Geometry,
}

impl<B, R, D> Co5300<B, R, D> {
    /// Creates a panel. No hardware is touched until [`Self::init`].
    pub const fn new(bus: B, reset: R, delay: D, geometry: Geometry) -> Self {
        Self {
            bus,
            reset,
            delay,
            geometry,
        }
    }
}

impl<B: Bus, R, D> Co5300<B, R, D> {
    /// Sets the address window and starts a memory write.
    fn set_window(&mut self, region: Region) -> Result<(), DisplayError<B::Error>> {
        if region.is_empty() || !region.is_within(self.geometry.width, self.geometry.height) {
            return Err(DisplayError::RegionOutOfBounds);
        }
        let x0 = region.x + self.geometry.column_offset;
        let y0 = region.y + self.geometry.row_offset;
        let x1 = x0 + region.width - 1;
        let y1 = y0 + region.height - 1;
        self.command_data(CMD_CASET, &[high(x0), low(x0), high(x1), low(x1)])?;
        self.command_data(CMD_PASET, &[high(y0), low(y0), high(y1), low(y1)])?;
        self.command(CMD_RAMWR)
    }

    /// Sends a parameter-less driver command.
    fn command(&mut self, command: u8) -> Result<(), DisplayError<B::Error>> {
        self.bus.write_command(command).map_err(DisplayError::Bus)
    }

    /// Sends a driver command with parameters.
    fn command_data(&mut self, command: u8, data: &[u8]) -> Result<(), DisplayError<B::Error>> {
        self.bus
            .write_command_data(command, data)
            .map_err(DisplayError::Bus)
    }
}

impl<B: Bus, R: OutputPin, D: DelayNs> Co5300<B, R, D> {
    /// Resets the panel and runs the vendor init table.
    pub fn init(&mut self) -> Result<(), DisplayError<B::Error>> {
        self.reset.set_high().map_err(|_| DisplayError::Reset)?;
        self.delay.delay_ms(RESET_PULSE_MS);
        self.reset.set_low().map_err(|_| DisplayError::Reset)?;
        self.delay.delay_ms(RESET_HOLD_MS);
        self.reset.set_high().map_err(|_| DisplayError::Reset)?;
        self.delay.delay_ms(RESET_HOLD_MS);

        self.command(CMD_SLPOUT)?;
        self.delay.delay_ms(SLPOUT_MS);

        self.command_data(CMD_WRITE_CMD_PAGE, &[CMD_PAGE_VENDOR])?;
        self.command_data(CMD_SPIMODECTL, &[SPIMODECTL_WRAM_ENABLE])?;
        self.command_data(CMD_PIXFMT, &[PIXFMT_RGB565])?;
        self.command_data(CMD_WCTRLD1, &[WCTRLD1_DEFAULT])?;
        self.command_data(CMD_BRIGHTNESS_HBM, &[BRIGHTNESS_HBM])?;
        self.command(CMD_DISPON)?;
        self.command_data(CMD_BRIGHTNESS, &[BRIGHTNESS_DEFAULT])?;
        self.command_data(CMD_WCE, &[WCE_OFF])?;
        self.delay.delay_ms(INIT_SETTLE_MS);

        self.command(CMD_INVOFF)?;
        self.command_data(CMD_MADCTL, &[MADCTL_RGB])
    }
}

impl<B: Bus, R, D: DelayNs> Display for Co5300<B, R, D> {
    type Error = DisplayError<B::Error>;

    fn width(&self) -> u16 {
        self.geometry.width
    }

    fn height(&self) -> u16 {
        self.geometry.height
    }

    fn display_on(&mut self) -> Result<(), Self::Error> {
        self.command(CMD_SLPOUT)?;
        self.delay.delay_ms(SLPOUT_MS);
        self.command(CMD_DISPON)
    }

    fn display_off(&mut self) -> Result<(), Self::Error> {
        self.command(CMD_DISPOFF)?;
        self.delay.delay_ms(SLPIN_MS);
        self.command(CMD_SLPIN)
    }

    fn set_brightness(&mut self, level: u8) -> Result<(), Self::Error> {
        self.command_data(CMD_BRIGHTNESS, &[level])
    }

    fn fill_region(&mut self, region: Region, color: Rgb565) -> Result<(), Self::Error> {
        if region.is_empty() {
            return Ok(());
        }
        let aligned = self.geometry.align_region(region);
        self.set_window(aligned)?;
        self.bus
            .fill_pixels(color, aligned.pixel_count())
            .map_err(DisplayError::Bus)
    }

    fn write_region(&mut self, region: Region, pixels: &[u8]) -> Result<(), Self::Error> {
        if pixels.len() != region.rgb565_len() {
            return Err(DisplayError::LengthMismatch);
        }
        if !self.geometry.is_aligned(region) {
            return Err(DisplayError::RegionMisaligned);
        }
        self.set_window(region)?;
        self.bus.write_pixels(pixels).map_err(DisplayError::Bus)
    }
}

fn high(value: u16) -> u8 {
    (value >> 8) as u8
}

fn low(value: u16) -> u8 {
    value as u8
}
