//! Shell session and awake mode state machine.
//!
//! Owns shell boot, presentation, live VBUS switching, and the per-mode awake
//! loop. Rendering is still the firmware presenter; the app view seam arrives
//! in 7.1.

use core::fmt::Debug;

use embassy_time::{Instant, Timer};
use esp_hal::peripherals::PSRAM;
use esp_println::println;
use hal::{Display, Power};
use power::PowerManager;

use common::{Event, LogEntry, Mode, Percent, Region, Timestamp};
use drivers::board;
use settings::{BootPolicy, Settings};
use shell::{Shell, ShellError};

use crate::boot::{self, VBUS_POLL_PERIOD};
use crate::bringup;
use crate::registry::{Registry, HOME_ID};

/// A fatal session failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunError {
    /// The power manager rejected a mode change.
    Power,
    /// The shell rejected a mode change.
    Shell,
}

/// The reason the watch awake loop stopped.
enum WatchExit {
    /// USB appeared while awake.
    VbusInserted,
    /// The awake budget expired.
    BudgetExpired,
}

/// Boots the shell with Home and runs its first lifecycle calls.
pub fn boot(
    mode: Mode,
    now: Timestamp,
    battery: Percent,
    is_charging: bool,
) -> Result<Shell<Registry>, ShellError> {
    let mut shell = Shell::new(
        Registry::new(),
        HOME_ID,
        mode,
        Region::full(board::LCD_WIDTH, board::LCD_HEIGHT),
    );
    shell.set_time(now);
    shell.set_battery(battery, is_charging);
    shell.start()?;
    shell.tick(0)?;
    Ok(shell)
}

/// Runs the awake mode state machine until watch mode should sleep.
///
/// Watch↔desk changes happen only through this function, so the rail plan,
/// shell mode, brightness, and retained mode-switch log stay in one place.
pub async fn run<D: Display, P: Power>(
    display: &mut D,
    shell: &mut Shell<Registry>,
    power: &mut PowerManager<P>,
    settings: &Settings,
    psram: &mut Option<PSRAM<'static>>,
    log: bool,
) -> Result<(), RunError>
where
    D::Error: Debug,
{
    let mut workspace = bringup::workspace();
    let base_now = shell.now();
    let started = Instant::now();
    let now = || {
        Timestamp::from_millis(
            base_now
                .as_millis()
                .saturating_add((Instant::now() - started).as_millis()),
        )
    };

    loop {
        shell.set_time(now());
        let mode = shell.mode();
        boot::apply_brightness(display, settings, mode, log);
        present(display, shell, settings, &mut workspace);
        match mode {
            Mode::Watch => {
                let awake_ms = BootPolicy::from_settings(settings).awake_budget_ms;
                match run_watch(power, shell, awake_ms, started).await {
                    WatchExit::VbusInserted => switch(shell, power, psram, Mode::Desk, log, now())?,
                    WatchExit::BudgetExpired => return Ok(()),
                }
            }
            Mode::Desk => {
                run_desk(power, shell, started).await;
                switch(shell, power, psram, Mode::Watch, log, now())?
            }
        }
    }
}

fn present<D: Display>(
    display: &mut D,
    shell: &Shell<Registry>,
    settings: &Settings,
    workspace: &mut bringup::Workspace,
) where
    D::Error: Debug,
{
    bringup::draw(display, shell.mode(), settings.display.theme, workspace);
}

fn switch<P: Power>(
    shell: &mut Shell<Registry>,
    power: &mut PowerManager<P>,
    psram: &mut Option<PSRAM<'static>>,
    target: Mode,
    log: bool,
    now: Timestamp,
) -> Result<(), RunError> {
    // Bring up PSRAM before the desk presenter allocates its gray surface.
    if target == Mode::Desk {
        if let Some(psram) = psram.take() {
            let bytes = boot::init_psram(psram);
            if log {
                println!("[boot] psram {} bytes", bytes);
            }
        }
    }

    // The presenter flushes synchronously, so no frame is pending here.
    let from = shell.mode();
    power.set_mode(target).map_err(|error| {
        println!("[pm] set_mode failed: {}", error.to_common());
        RunError::Power
    })?;
    shell.set_mode(target).map_err(|error| {
        println!("[shell] mode switch failed: {:?}", error);
        RunError::Shell
    })?;

    // Retained RAM only; watch mode never writes flash here.
    crate::retained::with_ring(|ring| {
        let _ = ring.push(&LogEntry::new(now, Event::ModeSwitch { from, to: target }));
    });
    if log {
        println!("[pm] switched {:?} -> {:?}", from, target);
    }
    Ok(())
}

async fn run_watch<P: Power>(
    power: &mut PowerManager<P>,
    shell: &mut Shell<Registry>,
    awake_ms: u32,
    started: Instant,
) -> WatchExit {
    let entered = Instant::now();
    loop {
        if power.is_vbus_present().unwrap_or(false) {
            return WatchExit::VbusInserted;
        }
        if (Instant::now() - entered).as_millis() >= u64::from(awake_ms) {
            return WatchExit::BudgetExpired;
        }
        Timer::after(VBUS_POLL_PERIOD).await;
        if let Err(error) = shell.tick((Instant::now() - started).as_millis()) {
            println!("[shell] tick failed: {:?}", error);
        }
    }
}

async fn run_desk<P: Power>(
    power: &mut PowerManager<P>,
    shell: &mut Shell<Registry>,
    started: Instant,
) {
    loop {
        Timer::after(VBUS_POLL_PERIOD).await;
        if !power.is_vbus_present().unwrap_or(true) {
            return;
        }
        if let Err(error) = shell.tick((Instant::now() - started).as_millis()) {
            println!("[shell] tick failed: {:?}", error);
        }
    }
}
