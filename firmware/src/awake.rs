//! Awake presentation: boot Home and run the shell for the active mode.

use core::fmt::Debug;

use embassy_time::{Duration, Timer};
use esp_hal::peripherals::PSRAM;
use esp_println::println;
use hal::{Display, Power};
use power::PowerManager;

use common::{Mode, Percent, Timestamp, WakeReason};
use settings::{BootPolicy, Settings, TimeFormat};
use shell::Shell;

use crate::boot::VBUS_POLL_PERIOD;
use crate::registry::Registry;
use crate::session;

/// An awake-session failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// The shell could not boot Home.
    Shell,
    /// The session failed while running.
    Session,
}

/// Everything one awake window needs.
pub struct Inputs<'a, D, P> {
    /// Active mode.
    pub mode: Mode,
    /// Wall-clock time.
    pub now: Timestamp,
    /// Battery percentage.
    pub battery: Percent,
    /// Charging state.
    pub is_charging: bool,
    /// Clock format.
    pub time_format: TimeFormat,
    /// Wake reason.
    pub wake: Option<WakeReason>,
    /// Due alarm, when one is ringing.
    pub alarm: Option<Timestamp>,
    /// Settings snapshot.
    pub settings: &'a Settings,
    /// Panel, when initialized.
    pub display: &'a mut Option<D>,
    /// Power manager.
    pub power: &'a mut PowerManager<P>,
    /// PSRAM token.
    pub psram: &'a mut Option<PSRAM<'static>>,
    /// Whether serial logging is enabled.
    pub log: bool,
    /// Alarm haptics, when enabled.
    pub haptics: Option<session::Haptics<'a>>,
}

/// Boots Home, runs the awake loop, and returns when watch should sleep.
pub async fn run<D: Display, P: Power>(input: Inputs<'_, D, P>) -> Result<(), Error>
where
    D::Error: Debug,
{
    let policy = BootPolicy::from_settings(input.settings);
    if let Some(reason) = input.wake {
        if let Err(error) = input.power.request_awake(reason, policy.awake_budget_ms, 0) {
            crate::retained::record_error(error.to_common().code());
            if input.log {
                println!("[pm] awake failed: {}", error.to_common());
            }
        }
    }

    let mut shell: Shell<Registry> = session::boot(
        input.mode,
        input.now,
        input.battery,
        input.is_charging,
        input.time_format,
        input.alarm,
    )
    .map_err(|error| {
        crate::retained::record_error(common::Error::NotReady.code());
        if input.log {
            println!("[shell] boot failed: {:?}", error);
        }
        Error::Shell
    })?;

    if let Some(panel) = input.display.as_mut() {
        session::run(session::RunInput {
            display: panel,
            shell: &mut shell,
            power: input.power,
            settings: input.settings,
            psram: input.psram,
            log: input.log,
            haptics: input.haptics,
        })
        .await
        .map_err(|error| {
            crate::retained::record_error(common::Error::NotReady.code());
            if input.log {
                println!("[session] failed: {:?}", error);
            }
            Error::Session
        })
    } else if input.mode == Mode::Desk {
        while input.power.is_vbus_present().unwrap_or(true) {
            Timer::after(VBUS_POLL_PERIOD).await;
        }
        Ok(())
    } else {
        Timer::after(Duration::from_millis(u64::from(policy.awake_budget_ms))).await;
        Ok(())
    }
}
