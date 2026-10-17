//! Host tests for the FT3168 driver against mock I2C, reset, and delay.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorKind, ErrorType, OutputPin};
use hal::Touch;

use crate::board::TP_I2C_ADDR;
use crate::test_support::{MockError, MockI2c};

use super::regs::*;
use super::{Ft3168, TouchError};

#[derive(Debug, Default)]
struct MockPin {
    states: Vec<bool>,
    fail: bool,
}

impl MockPin {
    fn failing() -> Self {
        Self {
            states: Vec::new(),
            fail: true,
        }
    }
}

#[derive(Debug)]
struct MockPinError;

impl embedded_hal::digital::Error for MockPinError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::Other
    }
}

impl ErrorType for MockPin {
    type Error = MockPinError;
}

impl OutputPin for MockPin {
    fn set_low(&mut self) -> Result<(), Self::Error> {
        if self.fail {
            return Err(MockPinError);
        }
        self.states.push(false);
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        if self.fail {
            return Err(MockPinError);
        }
        self.states.push(true);
        Ok(())
    }
}

#[derive(Debug, Default)]
struct MockDelay {
    elapsed_ms: u32,
}

impl DelayNs for MockDelay {
    fn delay_ns(&mut self, ns: u32) {
        self.elapsed_ms += ns / 1_000_000;
    }
}

fn driver() -> Ft3168<MockI2c, MockPin, MockDelay> {
    Ft3168::new(
        MockI2c::new(),
        TP_I2C_ADDR,
        MockPin::default(),
        MockDelay::default(),
    )
}

fn scripted() -> MockI2c {
    MockI2c::new()
        .with_reg(REG_TOUCH_DATA, 0x02)
        .with_reg(REG_TOUCH_DATA + 1, 0x01)
        .with_reg(REG_TOUCH_DATA + 2, 0x23)
        .with_reg(REG_TOUCH_DATA + 3, 0x01)
        .with_reg(REG_TOUCH_DATA + 4, 0x45)
        .with_reg(REG_TOUCH_DATA + 7, 0xFA)
        .with_reg(REG_TOUCH_DATA + 8, 0xBC)
        .with_reg(REG_TOUCH_DATA + 9, 0x57)
        .with_reg(REG_TOUCH_DATA + 10, 0x89)
}

#[test]
fn init_resets_then_starts_active_scanning() {
    let mut touch = driver();
    touch.init().unwrap();

    assert_eq!(touch.reset.states, [false, true]);
    assert!(touch.delay.elapsed_ms >= 105);
    assert_eq!(touch.i2c.reg(REG_POWER_MODE), POWER_ACTIVE);
}

#[test]
fn read_parses_two_points_in_one_burst() {
    let mut touch = driver();
    touch.i2c = scripted();

    let report = touch.read().unwrap();
    assert_eq!(report.count(), 2);
    let first = report.get(0).unwrap();
    assert_eq!((first.id, first.x, first.y), (0, 0x123, 0x145));
    let second = report.get(1).unwrap();
    assert_eq!((second.id, second.x, second.y), (5, 0xABC, 0x789));
}

#[test]
fn read_masks_the_coordinate_high_nibbles() {
    let mut touch = driver();
    touch.i2c = MockI2c::new()
        .with_reg(REG_TOUCH_DATA, 0x01)
        .with_reg(REG_TOUCH_DATA + 1, 0xF1)
        .with_reg(REG_TOUCH_DATA + 2, 0x23)
        .with_reg(REG_TOUCH_DATA + 3, 0xA2)
        .with_reg(REG_TOUCH_DATA + 4, 0x45);

    let report = touch.read().unwrap();
    let point = report.get(0).unwrap();
    assert_eq!((point.id, point.x, point.y), (0xA, 0x123, 0x245));
}

#[test]
fn no_fingers_is_an_empty_report() {
    let mut touch = driver();
    assert!(touch.read().unwrap().is_empty());
}

#[test]
fn count_is_clamped_to_the_hardware_limit() {
    let mut touch = driver();
    touch.i2c = scripted().with_reg(REG_TOUCH_DATA, 0x0F);

    let report = touch.read().unwrap();
    assert_eq!(report.count(), common::MAX_TOUCH_POINTS as u8);
    assert_eq!(report.points().len(), common::MAX_TOUCH_POINTS);
}

#[test]
fn chip_id_is_read_from_register_a0() {
    let mut touch = driver();
    touch.i2c = MockI2c::new().with_reg(REG_CHIP_ID, 0x68);
    assert_eq!(touch.read_chip_id().unwrap(), 0x68);
}

#[test]
fn sleep_enters_the_low_power_mode() {
    let mut touch = driver();
    touch.sleep().unwrap();
    assert_eq!(touch.i2c.reg(REG_POWER_MODE), POWER_SLEEP);
}

#[test]
fn wake_resets_and_returns_to_active() {
    let mut touch = driver();
    touch.sleep().unwrap();
    touch.wake().unwrap();

    assert_eq!(touch.reset.states, [false, true]);
    assert_eq!(touch.i2c.reg(REG_POWER_MODE), POWER_ACTIVE);
}

#[test]
fn bus_errors_are_surfaced() {
    let mut touch = driver();
    touch.i2c.set_fail(true);
    assert_eq!(touch.read().unwrap_err(), TouchError::Bus(MockError::Nack));
}

#[test]
fn reset_errors_are_surfaced() {
    let mut touch = Ft3168::new(
        MockI2c::new(),
        TP_I2C_ADDR,
        MockPin::failing(),
        MockDelay::default(),
    );
    assert_eq!(touch.init().unwrap_err(), TouchError::Reset);
}

#[test]
fn hal_trait_delegates_to_the_driver() {
    fn read_via_trait<T: Touch>(touch: &mut T) -> Result<common::TouchReport, T::Error> {
        touch.read()
    }

    let mut touch = driver();
    touch.i2c = scripted();
    let report = read_via_trait(&mut touch).unwrap();
    assert_eq!(report.count(), 2);
}
