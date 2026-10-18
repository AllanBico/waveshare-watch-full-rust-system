//! Temporary hardware preview: boot straight into the UI, before the shell is
//! wired into the firmware.
//!
//! The preview drives the real widget, layout, scene, damage, and flush paths.
//! It owns the live settings snapshot and persists user changes through the
//! boot-owned store. It has no app lifecycle. Delete it once the shell boots
//! through the firmware.
// TODO(phase-7): delete the preview layer and boot through Shell.

mod persistence;

use alloc::vec;
use core::fmt::Debug;

use embassy_time::{Duration, Instant, Timer};
use esp_println::println;
use static_cell::StaticCell;

use apps::{Overlay, Preview, Screen};
use common::{Mode, Percent, Region, RenderPolicy, Timestamp, TouchReport};
use drivers::board;
use hal::{Display, Power, Touch};
use power::PowerManager;
use settings::{Settings, SettingsStore};
use shell::{GestureOutcome, InputRouter, SwipeDirection, SwipeOrigin};
use ui::{flush, GraySurface, MonoSurface, Rgb565Source, Theme};

use crate::persist::SettingsBackend;
use crate::runtime::{INPUT_POLLS_PER_SECOND, INPUT_POLL_PERIOD};

/// Display staging buffer for chunked flushes.
const STAGING_BYTES: usize = 4096;

/// The watch framebuffer lives in internal SRAM.
const WATCH_FRAMEBUFFER_BYTES: usize = MonoSurface::buffer_len(board::LCD_WIDTH, board::LCD_HEIGHT);

static SESSION: StaticCell<Session> = StaticCell::new();

/// One preview session shared by the watch and desk loops.
struct Session {
    preview: Preview,
    input: InputRouter,
    mode: Mode,
    base_now: Timestamp,
    battery: Percent,
    is_charging: bool,
    staging: [u8; STAGING_BYTES],
    applied_brightness: Option<u8>,
}

impl Session {
    fn new(
        mode: Mode,
        now: Timestamp,
        battery: Percent,
        is_charging: bool,
        settings: Settings,
    ) -> Self {
        Self {
            preview: Preview::new(mode, now, battery, is_charging, settings),
            input: InputRouter::new(shell::GestureConfig::default()),
            mode,
            base_now: now,
            battery,
            is_charging,
            staging: [0; STAGING_BYTES],
            applied_brightness: None,
        }
    }

    fn step(&mut self, report: &TouchReport, uptime_ms: u64) -> bool {
        let was_settings = self.preview.screen() == Screen::Settings;
        let was_quick = self.preview.overlay() == Overlay::QuickSettings;
        let now = Timestamp::from_millis(self.base_now.as_millis().saturating_add(uptime_ms));
        self.preview.update(now, self.battery, self.is_charging);
        let outcome = self
            .input
            .handle(report, uptime_ms, board::LCD_WIDTH, board::LCD_HEIGHT);
        self.apply(outcome);
        let left_settings = was_settings && self.preview.screen() != Screen::Settings;
        let closed_quick = was_quick && self.preview.overlay() != Overlay::QuickSettings;
        left_settings || closed_quick
    }

    fn apply(&mut self, outcome: GestureOutcome) {
        if self.preview.screen() == Screen::Calibrate {
            match outcome {
                GestureOutcome::Tap { x, y } => {
                    self.preview.tap(x, y, viewport());
                }
                GestureOutcome::Swipe(swipe)
                    if swipe.origin == SwipeOrigin::LeftEdge
                        && swipe.direction == SwipeDirection::Right =>
                {
                    self.preview.back();
                }
                _ => {}
            }
            return;
        }
        match outcome {
            GestureOutcome::Tap { x, y } => {
                self.preview.tap(x, y, viewport());
            }
            GestureOutcome::Swipe(swipe) => self.swipe(swipe),
            GestureOutcome::None | GestureOutcome::Cancelled => {}
        }
    }

    fn swipe(&mut self, swipe: shell::Swipe) {
        if swipe.origin == SwipeOrigin::LeftEdge && swipe.direction == SwipeDirection::Right {
            self.preview.back();
            return;
        }
        if self.preview.screen() == Screen::Settings && self.preview.overlay() == Overlay::None {
            let delta = match swipe.direction {
                SwipeDirection::Up => 96,
                SwipeDirection::Down => -96,
                _ => 0,
            };
            if delta != 0 {
                self.preview.scroll_settings(delta);
                return;
            }
        }
        if swipe.direction == SwipeDirection::Down && self.preview.overlay() == Overlay::Drawer {
            self.preview.close_overlay();
            return;
        }
        if swipe.direction == SwipeDirection::Up && self.preview.overlay() == Overlay::QuickSettings
        {
            self.preview.close_overlay();
            return;
        }
        match (swipe.origin, swipe.direction) {
            (SwipeOrigin::BottomEdge, SwipeDirection::Up) => self.preview.open_drawer(),
            (SwipeOrigin::TopEdge, SwipeDirection::Down) => self.preview.open_quick_settings(),
            _ => {}
        }
    }

    fn render<D, S>(&mut self, display: &mut D, surface: &mut S)
    where
        D: Display,
        D::Error: Debug,
        S: ui::Surface + Rgb565Source,
    {
        if !self.preview.needs_render() {
            return;
        }
        let theme = Theme::for_id(
            self.preview.settings().display.theme,
            RenderPolicy::for_mode(self.mode).pixel,
        );
        match self.preview.render(surface, theme, viewport()) {
            Ok(damage) if damage.is_empty() => {}
            Ok(damage) => {
                if let Err(error) = flush(display, &*surface, &damage, &mut self.staging) {
                    self.preview.invalidate();
                    if self.mode == Mode::Desk {
                        println!("[ui] flush failed: {:?}", error);
                    }
                }
            }
            Err(error) => {
                self.preview.invalidate();
                if self.mode == Mode::Desk {
                    println!("[ui] render failed: {:?}", error);
                }
            }
        }
    }
}

fn viewport() -> Region {
    Region::full(board::LCD_WIDTH, board::LCD_HEIGHT)
}

/// Runs the preview in watch mode for the awake window.
pub async fn run_watch<D, T>(
    display: &mut D,
    touch: &mut T,
    now: Timestamp,
    battery: Percent,
    window: Duration,
    battery_mv: u16,
    settings: &mut SettingsStore<SettingsBackend>,
) where
    D: Display,
    D::Error: Debug,
    T: Touch,
{
    static FRAMEBUFFER: StaticCell<[u8; WATCH_FRAMEBUFFER_BYTES]> = StaticCell::new();
    let data = FRAMEBUFFER.init([0; WATCH_FRAMEBUFFER_BYTES]);
    let Ok(mut surface) = MonoSurface::new(board::LCD_WIDTH, board::LCD_HEIGHT, data) else {
        println!("[ui] watch framebuffer too small");
        return;
    };
    let session = SESSION.init(Session::new(
        Mode::Watch,
        now,
        battery,
        false,
        *settings.settings(),
    ));
    session.apply_brightness(display);
    session.render(display, &mut surface);

    let start = Instant::now();
    while Instant::now() - start < window {
        Timer::after(INPUT_POLL_PERIOD).await;
        let report = touch.read().unwrap_or(TouchReport::empty());
        let commit = session.step(&report, (Instant::now() - start).as_millis());
        session.sync_settings(settings);
        if commit {
            session.commit(settings, battery_mv);
        }
        session.apply_brightness(display);
        session.render(display, &mut surface);
    }
    session.sync_settings(settings);
    session.commit(settings, battery_mv);
}

/// Runs the preview in desk mode until USB is removed.
pub async fn run_desk<D, T, P>(
    display: &mut D,
    touch: &mut T,
    power: &mut PowerManager<P>,
    now: Timestamp,
    battery: Percent,
    battery_mv: u16,
    settings: &mut SettingsStore<SettingsBackend>,
) where
    D: Display,
    D::Error: Debug,
    T: Touch,
    P: Power,
{
    let mut data = vec![0u8; GraySurface::buffer_len(board::LCD_WIDTH, board::LCD_HEIGHT)];
    let Ok(mut surface) = GraySurface::new(board::LCD_WIDTH, board::LCD_HEIGHT, &mut data[..])
    else {
        println!("[ui] desk framebuffer too small");
        return;
    };
    let session = SESSION.init(Session::new(
        Mode::Desk,
        now,
        battery,
        true,
        *settings.settings(),
    ));
    session.apply_brightness(display);
    session.render(display, &mut surface);

    let start = Instant::now();
    let mut ticks = 0u32;
    loop {
        Timer::after(INPUT_POLL_PERIOD).await;
        let report = touch.read().unwrap_or(TouchReport::empty());
        let commit = session.step(&report, (Instant::now() - start).as_millis());
        session.sync_settings(settings);
        if commit {
            session.commit(settings, battery_mv);
        }
        session.apply_brightness(display);
        session.render(display, &mut surface);

        ticks += 1;
        if !ticks.is_multiple_of(INPUT_POLLS_PER_SECOND) {
            continue;
        }
        if !power.is_vbus_present().unwrap_or(true) {
            println!("[pm] USB removed");
            session.commit(settings, battery_mv);
            break;
        }
    }
    session.sync_settings(settings);
    session.commit(settings, battery_mv);
}
