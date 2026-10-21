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

use crate::desk::lab::{LabError, LabEvent, LabIntent};
use apps::{AUDIO_LAB_APP_ID, SETTINGS_APP_ID};
use common::{Event, LogEntry, Mode, Percent, Region, Timestamp, TouchReport};
use drivers::board;
use drivers::clock::{self, CpuSpeed};
use drivers::flash::FlashDevice;
use drivers::haptics::Motor;
use settings::{BootPolicy, Settings, SettingsEvent, SettingsStore};
use shell::{GestureOutcome, Shell, ShellError};
use static_cell::StaticCell;

use crate::boot::{self, VBUS_POLL_PERIOD};
use crate::log_boot::STORAGE_WRITE_MIN_MV;
use crate::log_view;
use crate::persist::SettingsBackend;
use crate::present;
use crate::registry::{Registry, HOME_ID};

/// Touch polling period while the watch is awake.
const TOUCH_POLL_PERIOD: Duration = Duration::from_millis(20);
/// How often the watch checks VBUS during an awake window.
const VBUS_CHECK_PERIOD: Duration = Duration::from_millis(500);
/// Periodic repaint while awake, so the clock still advances without touch.
const IDLE_REPAINT_PERIOD: Duration = Duration::from_millis(1_000);
/// How often desk samples CPU, RAM, and clock stats.
const STATS_PERIOD: Duration = Duration::from_millis(1_000);

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
    /// Settings store.
    pub settings: &'a mut SettingsStore<SettingsBackend>,
    /// Battery millivolts sampled at boot.
    pub battery_mv: u16,
    /// Flash device, when bring-up succeeded.
    pub device: Option<&'static FlashDevice>,
    /// Touch controller, when initialized.
    pub touch: &'a mut Option<boot::Touch>,
    /// PSRAM token, consumed on the first desk entry.
    pub psram: &'a mut Option<PSRAM<'static>>,
    /// Whether serial logging is enabled.
    pub log: bool,
    /// Alarm haptics, when an alarm is ringing.
    pub haptics: Option<Haptics<'a>>,
    /// Desk audio control, when this boot brought audio up.
    pub audio: Option<crate::desk::Control>,
    /// Desk peripherals, kept for the first desk entry after a watch boot.
    pub desk: Option<crate::desk::Inputs>,
    /// CPU1 lifecycle controller.
    pub core1: &'a mut crate::desk::Core1,
}

/// Inputs for one watch awake loop.
struct WatchInput<'a, 'h, D, P> {
    power: &'a mut PowerManager<P>,
    shell: &'a mut Shell<Registry>,
    settings: &'a mut SettingsStore<SettingsBackend>,
    battery_mv: u16,
    device: Option<&'static FlashDevice>,
    display: &'a mut D,
    touch: &'a mut Option<boot::Touch>,
    workspace: &'a mut present::Workspace,
    idle_ms: u32,
    awake_ms: u32,
    base_now: Timestamp,
    started: Instant,
    log: bool,
    haptics: Option<&'a mut Haptics<'h>>,
}

/// Inputs for one desk awake loop.
struct DeskInput<'a, D, P> {
    power: &'a mut PowerManager<P>,
    shell: &'a mut Shell<Registry>,
    settings: &'a mut SettingsStore<SettingsBackend>,
    battery_mv: u16,
    device: Option<&'static FlashDevice>,
    display: &'a mut D,
    touch: &'a mut Option<boot::Touch>,
    workspace: &'a mut present::Workspace,
    base_now: Timestamp,
    started: Instant,
    log: bool,
    audio: Option<&'a mut crate::desk::Control>,
}

/// The reason the watch awake loop stopped.
enum WatchExit {
    /// USB appeared while awake.
    VbusInserted,
    /// The awake budget expired.
    BudgetExpired,
}

/// The single shell for this boot, kept off the task stack.
static SHELL: StaticCell<Shell<Registry>> = StaticCell::new();

/// Boots the shell with Home and runs its first lifecycle calls.
pub fn boot(
    mode: Mode,
    now: Timestamp,
    battery: Percent,
    is_charging: bool,
    settings: &Settings,
    alarm: Option<Timestamp>,
) -> Result<&'static mut Shell<Registry>, ShellError> {
    let shell = SHELL.init_with(|| {
        let mut registry = Registry::new(mode, now, battery, is_charging, settings);
        registry.set_alarm(alarm);
        Shell::new(
            registry,
            HOME_ID,
            mode,
            Region::full(board::LCD_WIDTH, board::LCD_HEIGHT),
        )
    });
    shell.set_time(now);
    shell.set_battery(battery, is_charging);
    if mode == Mode::Desk {
        let _ = shell.set_system_stats(crate::metrics::snapshot());
        shell.registry_mut().ensure_audio_lab();
    }
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
        battery_mv,
        device,
        touch,
        psram,
        log,
        mut haptics,
        mut audio,
        mut desk,
        core1,
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
        boot::apply_brightness(display, settings.settings(), mode, log);
        let _ = present(display, shell, settings.settings(), &mut workspace, log)?;
        match mode {
            Mode::Watch => {
                let policy = BootPolicy::from_settings(settings.settings());
                let exit = run_watch(WatchInput {
                    power: &mut *power,
                    shell: &mut *shell,
                    settings: &mut *settings,
                    battery_mv,
                    device,
                    display: &mut *display,
                    touch: &mut *touch,
                    workspace: &mut workspace,
                    idle_ms: policy.idle_timeout_ms,
                    awake_ms: policy.awake_budget_ms,
                    base_now,
                    started,
                    log,
                    haptics: haptics.as_mut(),
                })
                .await;
                commit_settings(settings, battery_mv, log);
                match exit {
                    WatchExit::VbusInserted => {
                        haptics = None;
                        switch(shell, power, psram, Mode::Desk, log, now())?;
                        shell.registry_mut().ensure_audio_lab();
                        if let Some(inputs) = desk.take() {
                            audio = crate::desk::start(inputs, core1);
                        } else if let Some(audio) = audio.as_mut() {
                            if core1.resume() {
                                if let Err(error) = audio.init() {
                                    if log {
                                        println!("[audio] re-init failed: {:?}", error);
                                    }
                                }
                            } else if log {
                                println!("[audio] cpu1 resume failed");
                            }
                        }
                    }
                    WatchExit::BudgetExpired => return Ok(()),
                }
            }
            Mode::Desk => {
                run_desk(DeskInput {
                    power: &mut *power,
                    shell: &mut *shell,
                    settings: &mut *settings,
                    battery_mv,
                    device,
                    display: &mut *display,
                    touch: &mut *touch,
                    workspace: &mut workspace,
                    base_now,
                    started,
                    log,
                    audio: audio.as_mut(),
                })
                .await;
                if let Some(audio) = audio.as_mut() {
                    audio.shutdown().await;
                }
                core1.park();
                commit_settings(settings, battery_mv, log);
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
) -> Result<ui::Damage, RunError>
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
    // Desk runs at 240 MHz before PSRAM and the desk presenter are brought up.
    // Watch returns to 80 MHz after CPU1 has been parked.
    if target == Mode::Desk {
        clock::set_speed(CpuSpeed::Mhz240);
        if let Some(psram) = psram.take() {
            let bytes = boot::init_psram(psram);
            if log {
                println!("[boot] psram {} bytes", bytes);
            }
        }
    } else {
        clock::set_speed(CpuSpeed::Mhz80);
    }

    // The presenter flushes synchronously, so no frame is pending here.
    let from = shell.mode();
    power.set_mode(target).map_err(|error| {
        println!("[pm] set_mode failed: {}", error.to_common());
        RunError::Power
    })?;
    if let Err(error) = shell.set_mode(target) {
        // The shell rolls back to the previous mode on a failed Home entry, so
        // put the rails back to match it rather than leaving desk rails on.
        let restore = shell.mode();
        let restore_speed = if restore == Mode::Desk {
            CpuSpeed::Mhz240
        } else {
            CpuSpeed::Mhz80
        };
        clock::set_speed(restore_speed);
        if let Err(power_error) = power.set_mode(restore) {
            if log {
                println!("[pm] rollback failed: {}", power_error.to_common());
            }
            return Err(RunError::Power);
        }
        if log {
            println!("[shell] mode switch failed: {:?}", error);
        }
        return Err(RunError::Shell);
    }

    // Retained RAM only; watch mode never writes flash here.
    crate::retained::with_ring(|ring| {
        let _ = ring.push(&LogEntry::new(now, Event::ModeSwitch { from, to: target }));
    });
    if log {
        println!("[pm] switched {:?} -> {:?}", from, target);
    }
    Ok(())
}

async fn run_watch<'a, 'h, D: Display, P: Power>(input: WatchInput<'a, 'h, D, P>) -> WatchExit
where
    D::Error: Debug,
{
    const HAPTIC_WINDOW_MS: u64 = 1_000;

    let WatchInput {
        power,
        shell,
        settings,
        battery_mv,
        device,
        display,
        touch,
        workspace,
        idle_ms,
        awake_ms,
        base_now,
        started,
        log,
        mut haptics,
    } = input;
    let entered = Instant::now();
    let mut last_activity_ms = 0u64;
    let mut next_vbus = Instant::now() + VBUS_CHECK_PERIOD;
    let mut next_repaint = Instant::now() + IDLE_REPAINT_PERIOD;

    loop {
        let loop_now = Instant::now();
        let elapsed = (loop_now - entered).as_millis();
        if elapsed >= u64::from(awake_ms) {
            stop_haptics(&mut haptics);
            return WatchExit::BudgetExpired;
        }
        if elapsed.saturating_sub(last_activity_ms) >= u64::from(idle_ms) {
            stop_haptics(&mut haptics);
            return WatchExit::BudgetExpired;
        }
        if loop_now >= next_vbus {
            next_vbus = loop_now + VBUS_CHECK_PERIOD;
            if power.is_vbus_present().unwrap_or(false) {
                stop_haptics(&mut haptics);
                return WatchExit::VbusInserted;
            }
        }
        if let Some(haptics) = haptics.as_deref_mut() {
            let on = elapsed % HAPTIC_WINDOW_MS
                < u64::from(haptics.strength.value()) * HAPTIC_WINDOW_MS / 100;
            haptics.motor.set(on);
        }

        Timer::after(TOUCH_POLL_PERIOD).await;

        let uptime = (Instant::now() - started).as_millis();
        shell.set_time(Timestamp::from_millis(
            base_now.as_millis().saturating_add(uptime),
        ));
        let report = read_touch(touch);
        if !report.is_empty() {
            last_activity_ms = (Instant::now() - entered).as_millis();
        }
        let was_settings = shell.active_app() == Some(SETTINGS_APP_ID);
        let outcome = shell.handle_touch(&report, uptime).unwrap_or_else(|error| {
            crate::retained::record_error(common::Error::NotReady.code());
            if log {
                println!("[shell] touch failed: {:?}", error);
            }
            GestureOutcome::None
        });
        if let Err(error) = shell.tick(uptime) {
            crate::retained::record_error(common::Error::NotReady.code());
            if log {
                println!("[shell] tick failed: {:?}", error);
            }
        }
        let (changed, theme_changed) = sync_settings(shell, settings, log);
        if theme_changed {
            shell.invalidate_active();
        }
        let is_settings = shell.active_app() == Some(SETTINGS_APP_ID);
        if was_settings && !is_settings {
            commit_settings(settings, battery_mv, log);
        }
        if let Some(category) = shell.registry_mut().logs_mut().take_load_request() {
            log_view::load(category, device, shell.registry_mut().logs_mut(), log);
        }

        let completed = matches!(
            outcome,
            GestureOutcome::Tap { .. } | GestureOutcome::Swipe(_)
        );
        if completed || changed || Instant::now() >= next_repaint {
            next_repaint = Instant::now() + IDLE_REPAINT_PERIOD;
            boot::apply_brightness(display, settings.settings(), Mode::Watch, log);
            if let Err(error) = present(display, shell, settings.settings(), workspace, log) {
                crate::retained::record_error(common::Error::NotReady.code());
                if log {
                    println!("[ui] watch present failed: {:?}", error);
                }
            }
        }
    }
}

fn read_touch(touch: &mut Option<boot::Touch>) -> TouchReport {
    match touch.as_mut() {
        Some(touch) => touch.read().unwrap_or(TouchReport::empty()),
        None => TouchReport::empty(),
    }
}

fn sync_settings(
    shell: &mut Shell<Registry>,
    store: &mut SettingsStore<SettingsBackend>,
    log: bool,
) -> (bool, bool) {
    let mut changed = false;
    let mut theme_changed = false;
    while let Some((id, value)) = shell.registry_mut().pop_setting_intent() {
        match store.set(id, value) {
            Ok(event) => {
                changed = true;
                theme_changed |= matches!(event, SettingsEvent::Theme);
            }
            Err(error) => {
                if log {
                    println!("[set] rejected {}: {}", id.label(), error);
                }
            }
        }
    }
    if changed {
        let mode = shell.mode();
        shell.registry_mut().apply_settings(mode, store.settings());
    }
    (changed, theme_changed)
}

fn commit_settings(store: &mut SettingsStore<SettingsBackend>, battery_mv: u16, log: bool) {
    if battery_mv < STORAGE_WRITE_MIN_MV || !store.is_dirty() {
        return;
    }
    match store.flush() {
        Ok(count) if log && count > 0 => println!("[set] wrote {} keys", count),
        Ok(_) => {}
        Err(error) => {
            if log {
                println!("[set] flush failed: {}", error);
            }
        }
    }
}

fn report_control_error(shell: &mut Shell<Registry>) {
    if let Some(app) = shell.registry_mut().audio_lab_mut() {
        app.ingest(LabEvent::Error(LabError::Codec));
    }
}

fn service_audio(
    shell: &mut Shell<Registry>,
    mut audio: Option<&mut crate::desk::Control>,
    log: bool,
) -> bool {
    let mut changed = false;
    loop {
        let Some(intent) = shell
            .registry_mut()
            .audio_lab_mut()
            .and_then(|app| app.take_intent())
        else {
            break;
        };
        let Some(audio) = audio.as_deref_mut() else {
            if let Some(app) = shell.registry_mut().audio_lab_mut() {
                app.ingest(LabEvent::Error(LabError::NotReady));
            }
            changed = true;
            continue;
        };
        match intent {
            LabIntent::Command(command) => {
                if log {
                    println!("[audio] cmd {:?}", command);
                }
                changed |= crate::desk::lab::command(command);
            }
            LabIntent::SetMicGain(db) => {
                if audio.set_gain(db).is_err() {
                    report_control_error(shell);
                    changed = true;
                }
            }
            LabIntent::SetVolume(percent) => {
                if audio.set_volume(percent).is_err() {
                    report_control_error(shell);
                    changed = true;
                }
            }
            LabIntent::SetMute(muted) => {
                if audio.set_mute(muted).is_err() {
                    report_control_error(shell);
                    changed = true;
                }
            }
        }
    }
    while let Some(event) = crate::desk::lab::try_event() {
        if log
            && !matches!(
                event,
                LabEvent::Recording { .. } | LabEvent::Playback { .. }
            )
        {
            if matches!(event, LabEvent::Finished) {
                let (input, output) = crate::metrics::play_peaks();
                println!("[audio] play peaks in {} out {}", input, output);
            }
            println!("[audio] ev {:?}", event);
        }
        if let Some(app) = shell.registry_mut().audio_lab_mut() {
            app.ingest(event);
        }
        changed = true;
    }
    changed
}

fn stop_haptics<'a, 'h>(haptics: &'a mut Option<&'a mut Haptics<'h>>) {
    if let Some(haptics) = haptics.as_deref_mut() {
        haptics.motor.set(false);
    }
}

async fn run_desk<D: Display, P: Power>(input: DeskInput<'_, D, P>)
where
    D::Error: Debug,
{
    let DeskInput {
        power,
        shell,
        settings,
        battery_mv,
        device,
        display,
        touch,
        workspace,
        base_now,
        started,
        log,
        mut audio,
    } = input;
    let mut next_vbus = Instant::now() + VBUS_POLL_PERIOD;
    let mut next_meter = Instant::now();
    let mut next_repaint = Instant::now() + IDLE_REPAINT_PERIOD;
    let mut next_stats = Instant::now() + STATS_PERIOD;
    let mut sampler = crate::metrics::Sampler::new();

    loop {
        let loop_now = Instant::now();
        if loop_now >= next_vbus {
            next_vbus = loop_now + VBUS_POLL_PERIOD;
            if !power.is_vbus_present().unwrap_or(true) {
                return;
            }
        }

        Timer::after(TOUCH_POLL_PERIOD).await;

        let uptime = (Instant::now() - started).as_millis();
        shell.set_time(Timestamp::from_millis(
            base_now.as_millis().saturating_add(uptime),
        ));
        let mut stats_changed = false;
        if Instant::now() >= next_stats {
            next_stats = Instant::now() + STATS_PERIOD;
            stats_changed = shell.set_system_stats(sampler.sample());
        }
        let report = read_touch(touch);
        let was_settings = shell.active_app() == Some(SETTINGS_APP_ID);
        let was_audio = shell.active_app() == Some(AUDIO_LAB_APP_ID);
        let outcome = shell.handle_touch(&report, uptime).unwrap_or_else(|error| {
            crate::retained::record_error(common::Error::NotReady.code());
            if log {
                println!("[shell] touch failed: {:?}", error);
            }
            GestureOutcome::None
        });
        if let Err(error) = shell.tick(uptime) {
            crate::retained::record_error(common::Error::NotReady.code());
            if log {
                println!("[shell] tick failed: {:?}", error);
            }
        }
        let (changed, theme_changed) = sync_settings(shell, settings, log);
        if theme_changed {
            shell.invalidate_active();
        }
        let is_settings = shell.active_app() == Some(SETTINGS_APP_ID);
        if was_settings && !is_settings {
            commit_settings(settings, battery_mv, log);
        }
        if let Some(category) = shell.registry_mut().logs_mut().take_load_request() {
            log_view::load(category, device, shell.registry_mut().logs_mut(), log);
        }
        let is_audio = shell.active_app() == Some(AUDIO_LAB_APP_ID);
        if was_audio && !is_audio && audio.is_some() {
            crate::desk::lab::stop();
        }
        let audio_changed = service_audio(shell, audio.as_deref_mut(), log);
        let audio_active = shell
            .registry()
            .audio_lab()
            .is_some_and(|app| app.is_active());
        let meter_due = audio_active && Instant::now() >= next_meter;
        if meter_due {
            next_meter = Instant::now() + Duration::from_millis(100);
        }

        let completed = matches!(
            outcome,
            GestureOutcome::Tap { .. } | GestureOutcome::Swipe(_)
        );
        if completed
            || changed
            || stats_changed
            || audio_changed
            || meter_due
            || Instant::now() >= next_repaint
        {
            next_repaint = Instant::now() + IDLE_REPAINT_PERIOD;
            boot::apply_brightness(display, settings.settings(), Mode::Desk, log);
            let render_start = Instant::now();
            match present(display, shell, settings.settings(), workspace, log) {
                Ok(damage) => {
                    if log && damage.is_full() {
                        println!("[ui] desk full frame");
                    }
                }
                Err(error) => {
                    crate::retained::record_error(common::Error::NotReady.code());
                    if log {
                        println!("[ui] desk present failed: {:?}", error);
                    }
                }
            }
            let render_ms = (Instant::now() - render_start).as_millis();
            if log && render_ms > 150 {
                println!("[ui] desk frame {} ms", render_ms);
            }
        }
    }
}
