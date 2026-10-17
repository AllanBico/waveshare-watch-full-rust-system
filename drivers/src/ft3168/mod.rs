//! FT3168 self-capacitance touch controller.
//!
//! I2C touch driver for the Waveshare ESP32-S3-Touch-AMOLED-2.06. Owns the
//! hardware reset sequence, active/sleep power modes, chip-ID readback, and
//! the burst that returns all reported points in one consistent snapshot.
//!
//! The FT3168 reports at most two points, so the driver reads a fixed
//! status-plus-two-points window instead of issuing one transaction per field.
//! Gesture interpretation is not part of the driver; it belongs in the input
//! layer above [`hal::Touch`].

use common::{TouchPoint, TouchReport, MAX_TOUCH_POINTS};
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
use hal::Touch;

mod error;
mod regs;

#[cfg(test)]
mod tests;

pub use error::TouchError;

use regs::*;

/// Reset pulse width. The datasheet requires at least 5 ms.
const RESET_PULSE_MS: u32 = 5;
/// Time to the first report after reset. The datasheet requires at least 70 ms.
const RESET_SETTLE_MS: u32 = 100;

/// An FT3168 touch controller on a shared I2C bus.
pub struct Ft3168<I, R, D> {
    i2c: I,
    address: u8,
    reset: R,
    delay: D,
}

impl<I, R, D> Ft3168<I, R, D> {
    /// Creates a driver. No hardware is touched until [`Self::init`].
    pub const fn new(i2c: I, address: u8, reset: R, delay: D) -> Self {
        Self {
            i2c,
            address,
            reset,
            delay,
        }
    }
}

impl<I: I2c, R: OutputPin, D: DelayNs> Ft3168<I, R, D> {
    /// Resets the controller and starts active scanning.
    ///
    /// Monitor mode does not report coordinates, so active mode is required
    /// for the touch path. The chip ID is read separately because some
    /// FT3168 variants report unexpected values.
    pub fn init(&mut self) -> Result<(), TouchError<I::Error>> {
        self.reset()?;
        self.set_power_mode(POWER_ACTIVE)
    }

    /// Reads the chip identity for diagnostics.
    pub fn read_chip_id(&mut self) -> Result<u8, TouchError<I::Error>> {
        self.read_u8(REG_CHIP_ID)
    }

    /// Reads all reported touch points in one burst.
    pub fn read(&mut self) -> Result<TouchReport, TouchError<I::Error>> {
        let mut buffer = [0u8; TOUCH_BUFFER_LEN];
        self.i2c
            .write_read(self.address, &[REG_TOUCH_DATA], &mut buffer)
            .map_err(TouchError::Bus)?;

        let count = (buffer[BLOCK_STATUS] & TOUCH_COUNT_MASK) as usize;
        let count = count.min(MAX_TOUCH_POINTS);
        let mut points = [TouchPoint::new(0, 0, 0); MAX_TOUCH_POINTS];
        for (index, point) in points.iter_mut().take(count).enumerate() {
            let base = BLOCK_POINT_BASE + index * POINT_STRIDE;
            let x_high = buffer[base + BLOCK_X_HIGH] & COORD_HIGH_MASK;
            let y_high = buffer[base + BLOCK_Y_HIGH];
            let x = (u16::from(x_high) << 8) | u16::from(buffer[base + BLOCK_X_LOW]);
            let y =
                (u16::from(y_high & COORD_HIGH_MASK) << 8) | u16::from(buffer[base + BLOCK_Y_LOW]);
            *point = TouchPoint::new(y_high >> TOUCH_ID_SHIFT, x, y);
        }
        Ok(TouchReport::new(&points[..count]))
    }

    /// Stops scanning and turns off the analog circuits.
    ///
    /// The datasheet notes that I2C access can be disturbed while the chip is
    /// asleep and another slave is used on the same bus. Wake with
    /// [`Self::wake`], which uses the reset pin rather than I2C.
    pub fn sleep(&mut self) -> Result<(), TouchError<I::Error>> {
        self.set_power_mode(POWER_SLEEP)
    }

    /// Resets the controller and restores active scanning.
    pub fn wake(&mut self) -> Result<(), TouchError<I::Error>> {
        self.reset()?;
        self.set_power_mode(POWER_ACTIVE)
    }

    fn reset(&mut self) -> Result<(), TouchError<I::Error>> {
        self.reset.set_low().map_err(|_| TouchError::Reset)?;
        self.delay.delay_ms(RESET_PULSE_MS);
        self.reset.set_high().map_err(|_| TouchError::Reset)?;
        self.delay.delay_ms(RESET_SETTLE_MS);
        Ok(())
    }

    fn set_power_mode(&mut self, mode: u8) -> Result<(), TouchError<I::Error>> {
        self.i2c
            .write(self.address, &[REG_POWER_MODE, mode])
            .map_err(TouchError::Bus)
    }

    fn read_u8(&mut self, reg: u8) -> Result<u8, TouchError<I::Error>> {
        let mut buffer = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buffer)
            .map_err(TouchError::Bus)?;
        Ok(buffer[0])
    }
}

impl<I: I2c, R: OutputPin, D: DelayNs> Touch for Ft3168<I, R, D> {
    type Error = TouchError<I::Error>;

    fn read(&mut self) -> Result<TouchReport, Self::Error> {
        Ft3168::read(self)
    }

    fn sleep(&mut self) -> Result<(), Self::Error> {
        Ft3168::sleep(self)
    }

    fn wake(&mut self) -> Result<(), Self::Error> {
        Ft3168::wake(self)
    }
}
