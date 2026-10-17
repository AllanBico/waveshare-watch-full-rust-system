//! Events and wake reasons.

use crate::mode::Mode;
use crate::units::Millivolts;

/// Why the device woke from sleep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum WakeReason {
    /// The RTC timer expired.
    Timer,
    /// The button was pressed.
    Button,
    /// The IMU detected motion.
    Motion,
    /// USB was connected.
    Usb,
    /// The cause could not be determined.
    Unknown,
}

/// A state change worth recording or reacting to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Event {
    /// The device woke, and why.
    Wake(WakeReason),
    /// The runtime mode changed.
    ModeSwitch { from: Mode, to: Mode },
    /// A battery voltage sample.
    Battery(Millivolts),
    /// A stable error code worth recording.
    Error(u16),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wake_event_carries_reason() {
        assert_eq!(
            Event::Wake(WakeReason::Motion),
            Event::Wake(WakeReason::Motion)
        );
    }

    #[test]
    fn mode_switch_event_carries_endpoints() {
        let event = Event::ModeSwitch {
            from: Mode::Watch,
            to: Mode::Desk,
        };
        match event {
            Event::ModeSwitch { from, to } => {
                assert_eq!(from, Mode::Watch);
                assert_eq!(to, Mode::Desk);
            }
            _ => unreachable!("constructed a mode switch event"),
        }
    }

    #[test]
    fn wake_reasons_are_distinct() {
        assert_ne!(WakeReason::Timer, WakeReason::Motion);
    }

    #[test]
    fn battery_and_error_are_events() {
        assert_ne!(
            Event::Battery(Millivolts::new(3_700)),
            Event::Battery(Millivolts::new(3_800))
        );
        assert_eq!(Event::Error(2), Event::Error(2));
    }

    #[test]
    fn events_differ_by_variant() {
        assert_ne!(
            Event::Wake(WakeReason::Timer),
            Event::ModeSwitch {
                from: Mode::Watch,
                to: Mode::Desk,
            }
        );
    }
}
