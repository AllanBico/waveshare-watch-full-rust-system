//! Phase 6.1 first light.
//!
//! Draws one mode label directly through the real surface and flush path, with
//! no shell and no apps. This proves display, surface, and flush before the
//! shell boots.
// TODO(phase-6.3): replace with the shell-owned debug Home.

use core::fmt::Debug;

use alloc::vec;
use esp_println::println;
use hal::Display;
use static_cell::StaticCell;

use common::{Mode, Region, RenderPolicy, ThemeId};
use drivers::board;
use ui::{draw_boot_label, flush, Damage, GraySurface, MonoSurface, Painter, Rgb565Source, Theme};

/// Display staging buffer for chunked flushes.
const STAGING_BYTES: usize = 4096;

/// The watch framebuffer lives in internal SRAM.
const WATCH_FRAMEBUFFER_BYTES: usize = MonoSurface::buffer_len(board::LCD_WIDTH, board::LCD_HEIGHT);

static WATCH_FRAMEBUFFER: StaticCell<[u8; WATCH_FRAMEBUFFER_BYTES]> = StaticCell::new();
static STAGING: StaticCell<[u8; STAGING_BYTES]> = StaticCell::new();

/// Reusable flush staging storage.
pub fn staging() -> &'static mut [u8; STAGING_BYTES] {
    STAGING.init([0; STAGING_BYTES])
}

/// The label for a mode.
pub const fn label(mode: Mode) -> &'static str {
    match mode {
        Mode::Watch => "WATCH",
        Mode::Desk => "DESK",
    }
}

/// Paints the mode label and flushes it to the panel.
pub fn draw<D: Display>(display: &mut D, mode: Mode, theme_id: ThemeId, staging: &mut [u8])
where
    D::Error: Debug,
{
    let theme = Theme::for_id(theme_id, RenderPolicy::for_mode(mode).pixel);
    match mode {
        Mode::Watch => draw_mono(display, theme, staging),
        Mode::Desk => draw_gray(display, theme, staging),
    }
}

fn draw_mono<D: Display>(display: &mut D, theme: &Theme, staging: &mut [u8])
where
    D::Error: Debug,
{
    let data = WATCH_FRAMEBUFFER.init([0; WATCH_FRAMEBUFFER_BYTES]);
    let Ok(mut surface) = MonoSurface::new(board::LCD_WIDTH, board::LCD_HEIGHT, data) else {
        println!("[ui] watch framebuffer too small");
        return;
    };
    {
        let mut painter = Painter::new(&mut surface);
        draw_boot_label(&mut painter, label(Mode::Watch), theme);
    }
    flush_surface(display, &surface, staging);
}

fn draw_gray<D: Display>(display: &mut D, theme: &Theme, staging: &mut [u8])
where
    D::Error: Debug,
{
    let mut data = vec![0u8; GraySurface::buffer_len(board::LCD_WIDTH, board::LCD_HEIGHT)];
    let Ok(mut surface) = GraySurface::new(board::LCD_WIDTH, board::LCD_HEIGHT, &mut data) else {
        println!("[ui] desk framebuffer too small");
        return;
    };
    {
        let mut painter = Painter::new(&mut surface);
        draw_boot_label(&mut painter, label(Mode::Desk), theme);
    }
    flush_surface(display, &surface, staging);
}

fn flush_surface<D: Display, S: Rgb565Source>(display: &mut D, surface: &S, staging: &mut [u8])
where
    D::Error: Debug,
{
    let damage = Damage::full(Region::full(surface.width(), surface.height()));
    if let Err(error) = flush(display, surface, &damage, staging) {
        println!("[ui] first-light flush failed: {:?}", error);
    }
}
