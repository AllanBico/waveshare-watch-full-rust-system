//! Host tests for the CO5300 driver, using mock transport, pin, and delay.

use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{ErrorKind, ErrorType, OutputPin};
use hal::Display;

use super::{Bus, Co5300, DisplayError, Geometry};
use crate::co5300::regs::*;

/// A recorded bus transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Event {
    Command(u8),
    CommandData(u8, Vec<u8>),
    Pixels(Vec<u8>),
    Fill(u16, u32),
}

/// Transport error for the mock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MockBusError;

/// A bus that records every call.
#[derive(Debug, Default)]
struct MockBus {
    events: Vec<Event>,
}

impl Bus for MockBus {
    type Error = MockBusError;

    fn write_command(&mut self, command: u8) -> Result<(), Self::Error> {
        self.events.push(Event::Command(command));
        Ok(())
    }

    fn write_command_data(&mut self, command: u8, data: &[u8]) -> Result<(), Self::Error> {
        self.events.push(Event::CommandData(command, data.to_vec()));
        Ok(())
    }

    fn write_pixels(&mut self, pixels: &[u8]) -> Result<(), Self::Error> {
        self.events.push(Event::Pixels(pixels.to_vec()));
        Ok(())
    }

    fn fill_pixels(&mut self, color: common::Rgb565, count: u32) -> Result<(), Self::Error> {
        self.events.push(Event::Fill(color.raw(), count));
        Ok(())
    }
}

/// A reset pin that never fails.
#[derive(Debug, Default)]
struct MockPin;

/// Pin error for the mock.
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
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A delay that does nothing.
#[derive(Debug, Default)]
struct MockDelay;

impl DelayNs for MockDelay {
    fn delay_ns(&mut self, _ns: u32) {}
}

const GEOMETRY: Geometry = Geometry {
    width: 410,
    height: 502,
    column_offset: 22,
    row_offset: 0,
};

fn panel() -> Co5300<MockBus, MockPin, MockDelay> {
    Co5300::new(MockBus::default(), MockPin, MockDelay, GEOMETRY)
}

#[test]
fn init_sends_the_vendor_sequence() {
    let mut panel = panel();
    panel.init().unwrap();

    use Event::{Command, CommandData};
    assert_eq!(
        panel.bus.events,
        vec![
            Command(CMD_SLPOUT),
            CommandData(CMD_WRITE_CMD_PAGE, vec![CMD_PAGE_VENDOR]),
            CommandData(CMD_SPIMODECTL, vec![SPIMODECTL_WRAM_ENABLE]),
            CommandData(CMD_PIXFMT, vec![PIXFMT_RGB565]),
            CommandData(CMD_WCTRLD1, vec![WCTRLD1_DEFAULT]),
            CommandData(CMD_BRIGHTNESS_HBM, vec![BRIGHTNESS_HBM]),
            Command(CMD_DISPON),
            CommandData(CMD_BRIGHTNESS, vec![BRIGHTNESS_DEFAULT]),
            CommandData(CMD_WCE, vec![WCE_OFF]),
            Command(CMD_INVOFF),
            CommandData(CMD_MADCTL, vec![MADCTL_RGB]),
        ]
    );
}

#[test]
fn fill_region_sets_window_with_offsets_and_fills() {
    let mut panel = panel();
    panel
        .fill_region(common::Region::new(5, 6, 10, 20), common::Rgb565::RED)
        .unwrap();

    use Event::{Command, CommandData, Fill};
    assert_eq!(
        panel.bus.events,
        vec![
            CommandData(CMD_CASET, vec![0x00, 0x1B, 0x00, 0x24]),
            CommandData(CMD_PASET, vec![0x00, 0x06, 0x00, 0x19]),
            Command(CMD_RAMWR),
            Fill(common::Rgb565::RED.raw(), 200),
        ]
    );
}

#[test]
fn write_region_streams_bytes_into_the_window() {
    let mut panel = panel();
    let region = common::Region::new(0, 0, 2, 1);
    let pixels = [0xF8, 0x00, 0x07, 0xE0];
    panel.write_region(region, &pixels).unwrap();

    use Event::{Command, CommandData, Pixels};
    assert_eq!(
        panel.bus.events,
        vec![
            CommandData(CMD_CASET, vec![0x00, 0x16, 0x00, 0x17]),
            CommandData(CMD_PASET, vec![0x00, 0x00, 0x00, 0x00]),
            Command(CMD_RAMWR),
            Pixels(pixels.to_vec()),
        ]
    );
}

#[test]
fn write_region_rejects_a_length_mismatch() {
    let mut panel = panel();
    let region = common::Region::new(0, 0, 4, 4);
    let error = panel.write_region(region, &[0u8; 4]).unwrap_err();
    assert_eq!(error, DisplayError::LengthMismatch);
    assert!(panel.bus.events.is_empty());
}

#[test]
fn fill_region_rejects_out_of_bounds() {
    let mut panel = panel();
    let error = panel
        .fill_region(common::Region::new(400, 0, 20, 10), common::Rgb565::BLUE)
        .unwrap_err();
    assert_eq!(error, DisplayError::RegionOutOfBounds);
    assert!(panel.bus.events.is_empty());
}

#[test]
fn set_brightness_writes_the_normal_mode_register() {
    let mut panel = panel();
    panel.set_brightness(0x42).unwrap();
    assert_eq!(
        panel.bus.events,
        vec![Event::CommandData(CMD_BRIGHTNESS, vec![0x42])]
    );
}

#[test]
fn display_on_and_off_use_mipi_order() {
    let mut panel = panel();
    panel.display_on().unwrap();
    panel.display_off().unwrap();
    assert_eq!(
        panel.bus.events,
        vec![
            Event::Command(CMD_SLPOUT),
            Event::Command(CMD_DISPON),
            Event::Command(CMD_DISPOFF),
            Event::Command(CMD_SLPIN),
        ]
    );
}

#[test]
fn pin_errors_surface_as_reset() {
    struct FailingPin;
    impl ErrorType for FailingPin {
        type Error = MockPinError;
    }
    impl OutputPin for FailingPin {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            Err(MockPinError)
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Err(MockPinError)
        }
    }

    let mut panel = Co5300::new(MockBus::default(), FailingPin, MockDelay, GEOMETRY);
    assert_eq!(panel.init().unwrap_err(), DisplayError::Reset);
}

#[test]
fn error_kinds_map_to_common_codes() {
    use common::Error;
    assert_eq!(
        DisplayError::<MockBusError>::Reset.to_common(),
        Error::HardwareFailure
    );
    assert_eq!(
        DisplayError::<MockBusError>::RegionOutOfBounds.to_common(),
        Error::InvalidInput
    );
    assert_eq!(
        DisplayError::<MockBusError>::Bus(MockBusError).to_common(),
        Error::HardwareFailure
    );
}
