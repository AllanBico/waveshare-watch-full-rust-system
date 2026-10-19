//! Shell session and awake mode state machine.
//!
//! Owns shell boot, presentation, live VBUS switching, and the per-mode awake
//! loop. The active app paints through the shell; the firmware only owns the
//! surface and the flush.

use core::fmt::Debug;

use embassy_time::{Duration, Instant, Timer};
use esp_hal::peripherals::PSRAM;
use esp_println::println;
use hal::{Display, Power};
use power::PowerManager;

use common::{Event, LogEntry, Mode, Percent, Region, Timestamp};
use drivers::board;
use drivers::haptics::Motor;
use settings::{BootPolicy, Settings, TimeFormat};
use shell::{Shell, ShellError};

use crate::boot::{self, VBUS_POLL_PERIOD};
use crate::present;
use crate::registry::{Registry, HOME_ID};

/// A fatal session failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunError {
    /// The power manager rejected a mode change.
    Power,
    /// The shell rejected a mode change.
    Shell,
    /// The active app could not render or flush.
    Render,
}

/// Haptics driven during an alarm wake.
pub struct Haptics<'a> {
    /// Motor switch.
    pub motor: &'a mut Motor,
    /// Configured strength in percent.
    pub strength: Percent,
}

/// Everything one awake session needs.
pub struct RunInput<'a, D, P> {
    /// Panel.
    pub display: &'a mut D,
    /// Shell with the active app.
    pub shell: &'a mut Shell<Registry>,
    /// Power manager.
    pub power: &'a mut PowerManager<P>,
    /// Settings snapshot.
    pub settings: &'a Settings,
    /// PSRAM token, consumed on the first desk entry.
    pub psram: &'a mut Option<PSRAM<'static>>,
    /// Whether serial logging is enabled.
    pub log: bool,
    /// Alarm haptics, when an alarm is ringing.
    pub haptics: Option<Haptics<'a>>,
}

/// Inputs for one watch awake loop.
struct WatchInput<'a, 'h, P> {
    power: &'a mut PowerManager<P>,
    shell: &'a mut Shell<Registry>,
    awake_ms: u32,
    started: Instant,
    log: bool,
    haptics: Option<&'a mut Haptics<'h>>,
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
    time_format: TimeFormat,
    alarm: Option<Timestamp>,
) -> Result<Shell<Registry>, ShellError> {
    let mut registry = Registry::new(mode, now, battery, is_charging, time_format);
    registry.set_alarm(alarm);
    let mut shell = Shell::new(
        registry,
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
pub async fn run<D: Display, P: Power>(input: RunInput<'_, D, P>) -> Result<(), RunError>
where
    D::Error: Debug,
{
    let RunInput {
        display,
        shell,
        power,
        settings,
        psram,
        log,
        mut haptics,
    } = input;
    let mut workspace = present::workspace();
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
        present(display, shell, settings, &mut workspace, log)?;
        match mode {
            Mode::Watch => {
                let awake_ms = BootPolicy::from_settings(settings).awake_budget_ms;
                let exit = {
                    let watch = WatchInput {
                        power: &mut *power,
                        shell: &mut *shell,
                        awake_ms,
                        started,
                        log,
                        haptics: haptics.as_mut(),
                    };
                    run_watch(watch).await
                };
                match exit {
                    WatchExit::VbusInserted => {
                        haptics = None;
                        switch(shell, power, psram, Mode::Desk, log, now())?
                    }
                    WatchExit::BudgetExpired => return Ok(()),
                }
            }
            Mode::Desk => {
                run_desk(power, shell, started, log).await;
                switch(shell, power, psram, Mode::Watch, log, now())?
            }
        }
    }
}

fn present<D: Display>(
    display: &mut D,
    shell: &mut Shell<Registry>,
    settings: &Settings,
    workspace: &mut present::Workspace,
    log: bool,
) -> Result<(), RunError>
where
    D::Error: Debug,
{
    present::render(display, shell, settings.display.theme, workspace).map_err(|error| {
        if log {
            println!("[ui] render failed: {:?}", error);
        }
        RunError::Render
    })
}

fn switch<P: Power>(
    shell: &mut Shell<Registry>,
    power: &mut PowerManager<P>,
    psram: &mut Option<PSRAM<'static>>,
    target: Mode,
    log: bool,
    now: Timestamp,
) -> Result<(), RunError> {
    // Bring up the heap and PSRAM before the desk presenter allocates its gray
    // surface. Watch never initializes either.
    if target == Mode::Desk {
        boot::ensure_sram_heap();
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

async fn run_watch<'a, 'h, P: Power>(input: WatchInput<'a, 'h, P>) -> WatchExit {
    const ALARM_TICK_MS: u64 = 100;
    const HAPTIC_WINDOW_MS: u64 = 1_000;

    let WatchInput {
        power,
        shell,
        awake_ms,
        started,
        log,
        mut haptics,
    } = input;
    let entered = Instant::now();
    let period = if haptics.is_some() {
        Duration::from_millis(ALARM_TICK_MS)
    } else {
        VBUS_POLL_PERIOD
    };
    loop {
        if power.is_vbus_present().unwrap_or(false) {
            stop_haptics(&mut haptics);
            return WatchExit::VbusInserted;
        }
        let elapsed = (Instant::now() - entered).as_millis();
        if elapsed >= u64::from(awake_ms) {
            stop_haptics(&mut haptics);
            return WatchExit::BudgetExpired;
        }
        if let Some(haptics) = haptics.as_deref_mut() {
            let on = elapsed % HAPTIC_WINDOW_MS
                < u64::from(haptics.strength.value()) * HAPTIC_WINDOW_MS / 100;
            haptics.motor.set(on);
        }
        Timer::after(period).await;
        if let Err(error) = shell.tick((Instant::now() - started).as_millis()) {
            crate::retained::record_error(common::Error::NotReady.code());
            if log {
                println!("[shell] tick failed: {:?}", error);
            }
        }
    }
}

fn stop_haptics<'a, 'h>(haptics: &'a mut Option<&'a mut Haptics<'h>>) {
    if let Some(haptics) = haptics.as_deref_mut() {
        haptics.motor.set(false);
    }
}

async fn run_desk<P: Power>(
    power: &mut PowerManager<P>,
    shell: &mut Shell<Registry>,
    started: Instant,
    log: bool,
) {
    loop {
        Timer::after(VBUS_POLL_PERIOD).await;
        if !power.is_vbus_present().unwrap_or(true) {
            return;
        }
        if let Err(error) = shell.tick((Instant::now() - started).as_millis()) {
            crate::retained::record_error(common::Error::NotReady.code());
            if log {
                println!("[shell] tick failed: {:?}", error);
            }
        }
    }
}
