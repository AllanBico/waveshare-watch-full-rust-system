//! PCF85063A real-time clock driver.
//!
//! I2C wall clock: reads and writes BCD date/time, forces 24-hour mode,
//! disables the CLKOUT output for power, and surfaces the oscillator-stop
//! flag. Implements [`hal::Time`] in UTC.

use common::{is_supported_year, DateTime, Timestamp};
use embedded_hal::i2c::I2c;
use hal::Time;

mod error;
mod regs;

#[cfg(test)]
mod tests;

pub use error::RtcError;

use regs::{
    BLOCK_DAYS, BLOCK_HOURS, BLOCK_MINUTES, BLOCK_MONTHS, BLOCK_SECONDS, BLOCK_YEARS, CTRL1,
    CTRL1_12_HOUR, CTRL1_STOP, CTRL2, CTRL2_COF_MASK, CTRL2_COF_OFF, DAYS_MASK, HOURS_MASK,
    MINUTES_MASK, MONTHS_MASK, SECONDS, SECONDS_MASK, SECONDS_OS, TIME_BLOCK_LEN,
};

/// Base year for the RTC's two-digit year field.
const YEAR_BASE: u16 = 2000;

/// A PCF85063A real-time clock on a shared I2C bus.
pub struct Pcf85063a<I> {
    i2c: I,
    address: u8,
}

impl<I: I2c> Pcf85063a<I> {
    /// Creates a driver for the RTC at `address`.
    ///
    /// The chip has no identity register, so nothing is verified here; call
    /// [`Self::init`] to reset and configure it.
    pub const fn new(i2c: I, address: u8) -> Self {
        Self { i2c, address }
    }

    /// Configures 24-hour mode with CLKOUT disabled, preserving the time.
    ///
    /// A soft reset would clear the time registers on every boot, so init only
    /// touches the control registers. Use [`Self::set_datetime`] to seed a
    /// stopped oscillator.
    pub fn init(&mut self) -> Result<(), RtcError<I::Error>> {
        let ctrl1 = self.read_u8(CTRL1)? & !(CTRL1_STOP | CTRL1_12_HOUR);
        self.write_u8(CTRL1, ctrl1)?;
        let ctrl2 = self.read_u8(CTRL2)?;
        self.write_u8(CTRL2, (ctrl2 & !CTRL2_COF_MASK) | CTRL2_COF_OFF)
    }

    /// Reads the current UTC date/time.
    ///
    /// Returns [`RtcError::TimeInvalid`] if the oscillator-stop flag is set.
    pub fn read_datetime(&mut self) -> Result<DateTime, RtcError<I::Error>> {
        let mut buf = [0u8; TIME_BLOCK_LEN];
        self.i2c
            .write_read(self.address, &[SECONDS], &mut buf)
            .map_err(RtcError::Bus)?;
        if buf[BLOCK_SECONDS] & SECONDS_OS != 0 {
            return Err(RtcError::TimeInvalid);
        }
        Ok(DateTime::new(
            YEAR_BASE + u16::from(bcd_to_dec(buf[BLOCK_YEARS])),
            bcd_to_dec(buf[BLOCK_MONTHS] & MONTHS_MASK),
            bcd_to_dec(buf[BLOCK_DAYS] & DAYS_MASK),
            bcd_to_dec(buf[BLOCK_HOURS] & HOURS_MASK),
            bcd_to_dec(buf[BLOCK_MINUTES] & MINUTES_MASK),
            bcd_to_dec(buf[BLOCK_SECONDS] & SECONDS_MASK),
        ))
    }

    /// Sets the UTC date/time, validating the fields first.
    pub fn set_datetime(&mut self, datetime: &DateTime) -> Result<(), RtcError<I::Error>> {
        if !datetime.is_valid() || !is_supported_year(datetime.year) {
            return Err(RtcError::InvalidField);
        }
        let ctrl1 = self.read_u8(CTRL1)?;
        self.write_u8(CTRL1, ctrl1 | CTRL1_STOP)?;
        self.write_time_block(datetime)?;
        self.write_u8(CTRL1, ctrl1 & !CTRL1_STOP)
    }

    fn write_time_block(&mut self, datetime: &DateTime) -> Result<(), RtcError<I::Error>> {
        let block = [
            SECONDS,
            dec_to_bcd(datetime.second),
            dec_to_bcd(datetime.minute),
            dec_to_bcd(datetime.hour),
            dec_to_bcd(datetime.day),
            datetime.weekday(),
            dec_to_bcd(datetime.month),
            dec_to_bcd((datetime.year % 100) as u8),
        ];
        self.i2c.write(self.address, &block).map_err(RtcError::Bus)
    }

    fn read_u8(&mut self, reg: u8) -> Result<u8, RtcError<I::Error>> {
        let mut buf = [0u8; 1];
        self.i2c
            .write_read(self.address, &[reg], &mut buf)
            .map_err(RtcError::Bus)?;
        Ok(buf[0])
    }

    fn write_u8(&mut self, reg: u8, value: u8) -> Result<(), RtcError<I::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(RtcError::Bus)
    }
}

impl<I: I2c> Time for Pcf85063a<I> {
    type Error = RtcError<I::Error>;

    fn now(&mut self) -> Result<Timestamp, Self::Error> {
        self.read_datetime()?
            .to_timestamp()
            .ok_or(RtcError::InvalidField)
    }

    fn set(&mut self, timestamp: Timestamp) -> Result<(), Self::Error> {
        self.set_datetime(&DateTime::from_timestamp(timestamp))
    }
}

fn bcd_to_dec(bcd: u8) -> u8 {
    (bcd >> 4) * 10 + (bcd & 0x0F)
}

fn dec_to_bcd(dec: u8) -> u8 {
    ((dec / 10) << 4) | (dec % 10)
}
