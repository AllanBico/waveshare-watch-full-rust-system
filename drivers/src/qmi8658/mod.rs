//! QMI8658C six-axis IMU driver.
//!
//! I2C driver for the Waveshare ESP32-S3-Touch-AMOLED-2.06. Configures the
//! accelerometer and gyroscope, reads both in one burst, and configures
//! Wake-on-Motion on INT1 (initially low) for the deep-sleep wake path.
//!
//! Init does not issue a soft reset: the datasheet's reset-to-first-sample
//! time is 1.75 s. Instead, init disables the sensors and clears any previous
//! Wake-on-Motion configuration, which makes it safe to call on every wake.
//!
//! CTRL9 completion is polled on `STATUS1` bit 0 as the datasheet specifies.
//! Some vendor drivers use `STATUSINT` bit 7; verify on hardware if commands
//! time out.

use common::{Acceleration, AngularRate, MotionSample, WakeOnMotion};
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;
use hal::Imu;

mod error;
mod regs;

#[cfg(test)]
mod tests;

pub use error::ImuError;

use regs::*;

/// CTRL9 poll limit, in milliseconds.
const CTRL9_POLL_LIMIT_MS: u32 = 20;
/// Time for both sensors to produce settled data after enable.
const SENSOR_SETTLE_MS: u32 = 100;
/// Time for the accelerometer to settle in low-power WoM mode.
const ACCEL_SETTLE_MS: u32 = 30;

/// A QMI8658C IMU on a shared I2C bus.
pub struct Qmi8658<I, D> {
    i2c: I,
    address: u8,
    delay: D,
    accel_sensitivity: u16,
    gyro_sensitivity: u16,
}

impl<I, D> Qmi8658<I, D> {
    /// Creates a driver. No hardware is touched until [`Self::init`].
    pub const fn new(i2c: I, address: u8, delay: D) -> Self {
        Self {
            i2c,
            address,
            delay,
            accel_sensitivity: ACCEL_4G_LSB_PER_G,
            gyro_sensitivity: GYRO_512_LSB_PER_DPS,
        }
    }
}

impl<I: I2c, D: DelayNs> Qmi8658<I, D> {
    /// Probes the chip, clears any previous WoM state, and enables accel+gyro.
    pub fn init(&mut self) -> Result<(), ImuError<I::Error>> {
        let id = self.read_chip_id()?;
        if id != CHIP_ID {
            return Err(ImuError::BadChipId(id));
        }

        self.write_u8(CTRL1, CTRL1_INIT)?;
        self.write_u8(CTRL7, 0)?;
        self.clear_wake_on_motion()?;
        self.configure_normal()
    }

    /// Reads the chip identity.
    pub fn read_chip_id(&mut self) -> Result<u8, ImuError<I::Error>> {
        self.read_u8(WHO_AM_I)
    }

    /// Reads the chip revision.
    pub fn read_revision(&mut self) -> Result<u8, ImuError<I::Error>> {
        self.read_u8(REVISION_ID)
    }

    /// Reads acceleration and angular rate in one burst.
    pub fn read_motion(&mut self) -> Result<MotionSample, ImuError<I::Error>> {
        let mut buffer = [0u8; MOTION_LEN];
        self.read_seq(AX_L, &mut buffer)?;

        let acceleration = Acceleration::new(
            scale_accel(raw_i16(&buffer, 0), self.accel_sensitivity),
            scale_accel(raw_i16(&buffer, 2), self.accel_sensitivity),
            scale_accel(raw_i16(&buffer, 4), self.accel_sensitivity),
        );
        let angular_rate = AngularRate::new(
            scale_gyro(raw_i16(&buffer, 6), self.gyro_sensitivity),
            scale_gyro(raw_i16(&buffer, 8), self.gyro_sensitivity),
            scale_gyro(raw_i16(&buffer, 10), self.gyro_sensitivity),
        );
        Ok(MotionSample {
            acceleration,
            angular_rate,
        })
    }

    /// Configures Wake-on-Motion on INT1 with an initially low level.
    ///
    /// The accelerometer runs in low-power 128 Hz mode and the gyroscope is
    /// disabled, as the datasheet requires for low-power accelerometer ODRs.
    pub fn configure_wake_on_motion(
        &mut self,
        config: WakeOnMotion,
    ) -> Result<(), ImuError<I::Error>> {
        if !config.is_valid() {
            return Err(ImuError::InvalidConfig);
        }

        self.write_u8(CTRL7, 0)?;
        self.write_u8(CTRL2, ACCEL_FS_4G | ACCEL_ODR_LP_128HZ)?;
        self.write_u8(CTRL5, 0)?;
        self.write_u8(CAL1_L, config.threshold_mg)?;
        self.write_u8(
            CAL1_H,
            CAL1_H_INT1_INITIAL_LOW | (config.blanking_samples & CAL1_H_BLANKING_MASK),
        )?;
        self.send_command(CTRL9_WRITE_WOM_SETTING)?;
        self.write_u8(CTRL7, CTRL7_AEN)?;

        self.accel_sensitivity = ACCEL_4G_LSB_PER_G;
        self.delay.delay_ms(ACCEL_SETTLE_MS);
        Ok(())
    }

    /// Reads and clears the Wake-on-Motion event flag.
    pub fn take_wake_event(&mut self) -> Result<bool, ImuError<I::Error>> {
        Ok(self.read_u8(STATUS1)? & STATUS1_WOM != 0)
    }

    fn configure_normal(&mut self) -> Result<(), ImuError<I::Error>> {
        self.write_u8(CTRL2, ACCEL_FS_4G | ACCEL_ODR_125HZ)?;
        self.write_u8(CTRL3, GYRO_FS_512DPS | GYRO_ODR_125HZ)?;
        self.write_u8(CTRL5, 0)?;
        self.write_u8(CTRL7, CTRL7_AEN | CTRL7_GEN)?;

        self.accel_sensitivity = ACCEL_4G_LSB_PER_G;
        self.gyro_sensitivity = GYRO_512_LSB_PER_DPS;
        self.delay.delay_ms(SENSOR_SETTLE_MS);
        Ok(())
    }

    fn clear_wake_on_motion(&mut self) -> Result<(), ImuError<I::Error>> {
        self.write_u8(CAL1_L, 0)?;
        self.write_u8(CAL1_H, 0)?;
        self.send_command(CTRL9_WRITE_WOM_SETTING)
    }

    fn send_command(&mut self, command: u8) -> Result<(), ImuError<I::Error>> {
        self.write_u8(CTRL9, command)?;
        if !self.wait_command_done()? {
            return Err(ImuError::Timeout);
        }
        self.write_u8(CTRL9, CTRL9_NOP)?;
        // Newer firmware leaves STATUSINT bit 7 set until NOP is processed.
        let _ = self.wait_statusint_clear()?;
        Ok(())
    }

    fn wait_command_done(&mut self) -> Result<bool, ImuError<I::Error>> {
        for _ in 0..CTRL9_POLL_LIMIT_MS {
            if self.read_u8(STATUSINT)? & STATUSINT_CMD_DONE != 0 {
                return Ok(true);
            }
            if self.read_u8(STATUS1)? & STATUS1_CMD_DONE != 0 {
                return Ok(true);
            }
            self.delay.delay_ms(1);
        }
        Ok(false)
    }

    fn wait_statusint_clear(&mut self) -> Result<bool, ImuError<I::Error>> {
        for _ in 0..CTRL9_POLL_LIMIT_MS {
            if self.read_u8(STATUSINT)? & STATUSINT_CMD_DONE == 0 {
                return Ok(true);
            }
            self.delay.delay_ms(1);
        }
        Ok(false)
    }

    fn read_u8(&mut self, reg: u8) -> Result<u8, ImuError<I::Error>> {
        let mut buffer = [0u8; 1];
        self.read_seq(reg, &mut buffer)?;
        Ok(buffer[0])
    }

    fn read_seq(&mut self, reg: u8, buffer: &mut [u8]) -> Result<(), ImuError<I::Error>> {
        self.i2c
            .write_read(self.address, &[reg], buffer)
            .map_err(ImuError::Bus)
    }

    fn write_u8(&mut self, reg: u8, value: u8) -> Result<(), ImuError<I::Error>> {
        self.i2c
            .write(self.address, &[reg, value])
            .map_err(ImuError::Bus)
    }
}

impl<I: I2c, D: DelayNs> Imu for Qmi8658<I, D> {
    type Error = ImuError<I::Error>;

    fn read_motion(&mut self) -> Result<MotionSample, Self::Error> {
        Qmi8658::read_motion(self)
    }

    fn configure_wake_on_motion(&mut self, config: WakeOnMotion) -> Result<(), Self::Error> {
        Qmi8658::configure_wake_on_motion(self, config)
    }

    fn take_wake_event(&mut self) -> Result<bool, Self::Error> {
        Qmi8658::take_wake_event(self)
    }
}

fn raw_i16(buffer: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes([buffer[offset], buffer[offset + 1]])
}

fn scale_accel(raw: i16, sensitivity: u16) -> i16 {
    let value = i32::from(raw) * 1000 / i32::from(sensitivity);
    value.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
}

fn scale_gyro(raw: i16, sensitivity: u16) -> i32 {
    i32::from(raw) * 1000 / i32::from(sensitivity)
}
