//! Temporary hardware preview: boot straight into the UI, before the shell is
//! wired into the firmware.
//!
//! The preview drives the real widget, layout, scene, damage, and flush paths
//! with local visual state only. It has no app lifecycle, no persistence, and
//! no backend. Delete it once the shell boots through the firmware.
// TODO(phase-7): delete the preview layer and boot through Shell.

use alloc::vec;
use core::fmt::Debug;

use embassy_time::{Duration, Instant, Timer};
use esp_println::println;
use static_cell::StaticCell;

use apps::{Overlay, Preview};
use common::{Mode, Percent, Region, Timestamp, TouchReport};
use drivers::board;
use hal::{Display, Power, Touch};
use power::PowerManager;
use shell::{GestureOutcome, InputRouter, SwipeDirection, SwipeOrigin};
use ui::{flush, GraySurface, MonoSurface, Rgb565Source, Theme};

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
    theme: &'static Theme,
    mode: Mode,
    base_now: Timestamp,
    battery: Percent,
    is_charging: bool,
    staging: [u8; STAGING_BYTES],
}

impl Session {
    fn new(mode: Mode, now: Timestamp, battery: Percent, is_charging: bool) -> Self {
        let theme = match mode {
            Mode::Watch => &Theme::WATCH,
            Mode::Desk => &Theme::DESK,
        };
        Self {
            preview: Preview::new(mode, now, battery, is_charging),
            input: InputRouter::new(shell::GestureConfig::default()),
            theme,
            mode,
            base_now: now,
            battery,
            is_charging,
            staging: [0; STAGING_BYTES],
        }
    }

    fn step(&mut self, report: &TouchReport, uptime_ms: u64) {
        let now = Timestamp::from_millis(self.base_now.as_millis().saturating_add(uptime_ms));
        self.preview.update(now, self.battery, self.is_charging);
        let outcome = self.input.handle(report, uptime_ms, board::LCD_HEIGHT);
        self.apply(outcome);
    }

    fn apply(&mut self, outcome: GestureOutcome) {
        match outcome {
            GestureOutcome::Tap { x, y } => {
                self.preview.tap(x, y, viewport());
            }
            GestureOutcome::Swipe(swipe) => self.swipe(swipe),
            GestureOutcome::None | GestureOutcome::Cancelled => {}
        }
    }

    fn swipe(&mut self, swipe: shell::Swipe) {
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
        match self.preview.render(surface, self.theme, viewport()) {
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
    let session = SESSION.init(Session::new(Mode::Watch, now, battery, false));
    session.render(display, &mut surface);

    let start = Instant::now();
    while Instant::now() - start < window {
        Timer::after(INPUT_POLL_PERIOD).await;
        let report = touch.read().unwrap_or(TouchReport::empty());
        session.step(&report, (Instant::now() - start).as_millis());
        session.render(display, &mut surface);
    }
}

/// Runs the preview in desk mode until USB is removed.
pub async fn run_desk<D, T, P>(
    display: &mut D,
    touch: &mut T,
    power: &mut PowerManager<P>,
    now: Timestamp,
    battery: Percent,
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
    let session = SESSION.init(Session::new(Mode::Desk, now, battery, true));
    session.render(display, &mut surface);

    let start = Instant::now();
    let mut ticks = 0u32;
    loop {
        Timer::after(INPUT_POLL_PERIOD).await;
        let report = touch.read().unwrap_or(TouchReport::empty());
        session.step(&report, (Instant::now() - start).as_millis());
        session.render(display, &mut surface);

        ticks += 1;
        if !ticks.is_multiple_of(INPUT_POLLS_PER_SECOND) {
            continue;
        }
        if !power.is_vbus_present().unwrap_or(true) {
            println!("[pm] USB removed");
            break;
        }
    }
}
