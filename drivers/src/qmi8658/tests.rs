//! Host tests for the QMI8658C driver against mock I2C and delay.

use embedded_hal::delay::DelayNs;
use hal::Imu;

use common::{Acceleration, AngularRate, MotionSample, WakeOnMotion};

use crate::board::IMU_I2C_ADDR;
use crate::test_support::{MockError, MockI2c};

use super::regs::*;
use super::{ImuError, Qmi8658, SENSOR_SETTLE_MS};

#[derive(Debug, Default)]
struct MockDelay {
    elapsed_ms: u32,
}

impl DelayNs for MockDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.elapsed_ms += ns / 1_000_000;
    }
}

fn probed() -> Qmi8658<MockI2c, MockDelay> {
    let mock = MockI2c::new()
        .with_reg(WHO_AM_I, CHIP_ID)
        .with_reg(STATUS1, STATUS1_CMD_DONE);
    Qmi8658::new(mock, IMU_I2C_ADDR, MockDelay::default())
}

fn motion_registers() -> MockI2c {
    MockI2c::new()
        .with_reg(WHO_AM_I, CHIP_ID)
        .with_reg(STATUS1, STATUS1_CMD_DONE)
        .with_reg(AX_L, 0x00)
        .with_reg(AX_L + 1, 0x20)
        .with_reg(AX_L + 2, 0x00)
        .with_reg(AX_L + 3, 0xE0)
        .with_reg(AX_L + 4, 0x00)
        .with_reg(AX_L + 5, 0x10)
        .with_reg(GX_L, 0x40)
        .with_reg(GX_L + 1, 0x00)
        .with_reg(GX_L + 2, 0xC0)
        .with_reg(GX_L + 3, 0xFF)
        .with_reg(GX_L + 4, 0x80)
        .with_reg(GX_L + 5, 0x00)
}

#[test]
fn init_probes_and_configures_accel_and_gyro() {
    let mut imu = probed();
    imu.init().unwrap();

    assert_eq!(imu.read_chip_id().unwrap(), CHIP_ID);
    assert_eq!(imu.i2c.reg(CTRL1), CTRL1_INIT);
    assert_eq!(imu.i2c.reg(CTRL2), ACCEL_FS_4G | ACCEL_ODR_125HZ);
    assert_eq!(imu.i2c.reg(CTRL3), GYRO_FS_512DPS | GYRO_ODR_125HZ);
    assert_eq!(imu.i2c.reg(CTRL5), 0);
    assert_eq!(imu.i2c.reg(CTRL7), CTRL7_AEN | CTRL7_GEN);
    assert_eq!(imu.i2c.reg(CAL1_L), 0);
    assert_eq!(imu.i2c.reg(CAL1_H), 0);
    assert!(imu.delay.elapsed_ms >= SENSOR_SETTLE_MS);
}

#[test]
fn init_rejects_the_wrong_chip() {
    let mock = MockI2c::new().with_reg(WHO_AM_I, 0x00);
    let mut imu = Qmi8658::new(mock, IMU_I2C_ADDR, MockDelay::default());
    assert_eq!(imu.init().unwrap_err(), ImuError::BadChipId(0));
}

#[test]
fn reads_scaled_motion_in_one_burst() {
    let mut imu = Qmi8658::new(motion_registers(), IMU_I2C_ADDR, MockDelay::default());
    imu.init().unwrap();

    assert_eq!(
        imu.read_motion().unwrap(),
        MotionSample {
            acceleration: Acceleration::new(1000, -1000, 500),
            angular_rate: AngularRate::new(1000, -1000, 2000),
        }
    );
}

#[test]
fn configures_wake_on_motion_on_int1() {
    let mut imu = probed();
    imu.init().unwrap();
    imu.configure_wake_on_motion(WakeOnMotion::new(100, 10))
        .unwrap();

    assert_eq!(imu.i2c.reg(CTRL2), ACCEL_FS_4G | ACCEL_ODR_LP_128HZ);
    assert_eq!(imu.i2c.reg(CAL1_L), 100);
    assert_eq!(imu.i2c.reg(CAL1_H), 10);
    assert_eq!(imu.i2c.reg(CTRL7), CTRL7_AEN);
    assert_eq!(imu.i2c.reg(CTRL9), CTRL9_NOP);
}

#[test]
fn invalid_wake_config_is_rejected_before_writing() {
    let mut imu = probed();
    imu.init().unwrap();
    let writes = imu.i2c.writes().len();

    assert_eq!(
        imu.configure_wake_on_motion(WakeOnMotion::new(0, 0))
            .unwrap_err(),
        ImuError::InvalidConfig
    );
    assert_eq!(
        imu.configure_wake_on_motion(WakeOnMotion::new(100, 64))
            .unwrap_err(),
        ImuError::InvalidConfig
    );
    assert_eq!(imu.i2c.writes().len(), writes);
}

#[test]
fn init_accepts_statusint_completion() {
    let mock = MockI2c::new()
        .with_reg(WHO_AM_I, CHIP_ID)
        .with_reg(STATUSINT, STATUSINT_CMD_DONE);
    let mut imu = Qmi8658::new(mock, IMU_I2C_ADDR, MockDelay::default());
    imu.init().unwrap();
    assert_eq!(imu.i2c.reg(CTRL7), CTRL7_AEN | CTRL7_GEN);
}

#[test]
fn ctrl9_completion_times_out() {
    let mut imu = probed();
    imu.init().unwrap();
    imu.i2c.set_reg(STATUS1, 0);

    assert_eq!(
        imu.configure_wake_on_motion(WakeOnMotion::new(100, 10))
            .unwrap_err(),
        ImuError::Timeout
    );
}

#[test]
fn wake_event_reads_and_clears_status1() {
    let mut imu = probed();
    imu.init().unwrap();

    imu.i2c.set_reg(STATUS1, STATUS1_WOM);
    assert!(imu.take_wake_event().unwrap());

    imu.i2c.set_reg(STATUS1, 0);
    assert!(!imu.take_wake_event().unwrap());
}

#[test]
fn bus_errors_are_surfaced() {
    let mut imu = probed();
    imu.init().unwrap();
    imu.i2c.set_fail(true);

    assert_eq!(
        imu.read_motion().unwrap_err(),
        ImuError::Bus(MockError::Nack)
    );
}

#[test]
fn hal_trait_delegates_to_the_driver() {
    fn read_via_trait<T: Imu>(imu: &mut T) -> Result<MotionSample, T::Error> {
        imu.read_motion()
    }

    let mut imu = Qmi8658::new(motion_registers(), IMU_I2C_ADDR, MockDelay::default());
    imu.init().unwrap();
    assert_eq!(read_via_trait(&mut imu).unwrap().acceleration.x_mg, 1000);
}
