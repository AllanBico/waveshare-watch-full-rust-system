//! Per-mode boot capabilities.
//!
//! `main` runs one boot chain. This module names the only steps that differ
//! between watch and desk, so there are no duplicate `watch_init`/`desk_init`
//! bodies.

use core::fmt::Debug;

use embassy_time::Duration;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO9, PSRAM};
use esp_println::println;
use hal::Display;

use common::Mode;
use drivers::board;
use drivers::ft3168::Ft3168;
use drivers::qmi8658::Qmi8658;
use settings::Settings;

use crate::runtime::Bus;

/// Touch driver type owned by the boot chain.
pub type Touch = Ft3168<Bus, Output<'static>, Delay>;

/// IMU driver type owned by the boot chain.
pub type Imu = Qmi8658<Bus, Delay>;

/// SRAM heap size reserved for the runtime.
pub const SRAM_HEAP_BYTES: usize = 96 * 1024;

/// How long to stay awake in desk mode after USB is removed.
// TODO(phase-7): return to the 2 s watch budget once the wake cycle is tuned.
pub const AWAKE_WINDOW: Duration = Duration::from_secs(7);

/// How often desk mode checks whether USB was removed.
pub const VBUS_POLL_PERIOD: Duration = Duration::from_millis(500);

/// Deep-sleep timer interval. Longer for wake-on-motion debugging.
pub const SLEEP_TIMER_MS: u32 = 30_000;

/// Accelerometer samples ignored after entering Wake-on-Motion.
pub const WOM_BLANKING_SAMPLES: u8 = 10;

/// The boot steps that differ between modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// Desk initializes PSRAM; watch never touches it.
    pub initialize_psram: bool,
    /// Desk initializes touch; watch leaves it alone.
    pub initialize_touch: bool,
    /// Desk logs over serial; watch stays quiet.
    pub logging: bool,
}

impl Plan {
    /// Builds the capability plan for `mode`.
    pub const fn for_mode(mode: Mode) -> Self {
        match mode {
            Mode::Watch => Self {
                initialize_psram: false,
                initialize_touch: false,
                logging: false,
            },
            Mode::Desk => Self {
                initialize_psram: true,
                initialize_touch: true,
                logging: true,
            },
        }
    }
}

/// Applies the active mode's brightness setting to the panel.
pub fn apply_brightness<D: Display>(display: &mut D, settings: &Settings, mode: Mode, log: bool)
where
    D::Error: Debug,
{
    let level = (u16::from(settings.brightness(mode).value()) * 255 / 100) as u8;
    if let Err(error) = display.set_brightness(level) {
        if log {
            println!("[lcd] brightness failed: {:?}", error);
        }
    }
}

/// Initializes PSRAM for desk mode and returns its size.
pub fn init_psram(psram: PSRAM<'static>) -> usize {
    let (_, bytes) = esp_hal::psram::psram_raw_parts(&psram);
    esp_alloc::psram_allocator!(psram, esp_hal::psram);
    bytes
}

/// Creates and starts the touch controller for desk mode.
pub fn init_touch(bus: Bus, reset: GPIO9<'static>, log: bool) -> Touch {
    let mut touch = Ft3168::new(
        bus,
        board::TP_I2C_ADDR,
        Output::new(reset, Level::Low, OutputConfig::default()),
        Delay::new(),
    );
    if let Err(error) = touch.init() {
        if log {
            println!("[tp] init failed: {}", error);
        }
    }
    if log {
        match touch.read_chip_id() {
            Ok(id) => println!("[tp] FT3168 id 0x{:02X}", id),
            Err(_) => println!("[tp] FT3168 id unreadable"),
        }
    }
    touch
}

/// Initializes the IMU and returns its stable error code, if any.
pub fn init_imu(imu: &mut Imu, log: bool) -> Option<u16> {
    let error = match imu.init() {
        Ok(()) => None,
        Err(error) => {
            if log {
                println!("[imu] init failed: {}", error);
            }
            Some(error.to_common().code())
        }
    };
    if log {
        match (imu.read_chip_id().ok(), imu.read_revision().ok()) {
            (Some(id), Some(revision)) => {
                println!("[imu] QMI8658 id 0x{:02X} rev 0x{:02X}", id, revision)
            }
            _ => println!("[imu] identity unreadable"),
        }
    }
    error
}
