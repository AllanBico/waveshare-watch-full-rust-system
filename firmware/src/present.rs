//! Shell presentation: render the active app into the mode's surface and flush
//! only the damage it reports.

use core::fmt::Debug;

use alloc::vec::Vec;
use hal::Display;
use static_cell::StaticCell;

use common::{RenderPolicy, ThemeId};
use drivers::board;
use shell::Shell;
use ui::{flush, GraySurface, MonoSurface, Theme};

use crate::registry::Registry;

/// Display staging buffer for chunked flushes.
const STAGING_BYTES: usize = 4096;

/// The watch framebuffer lives in internal SRAM.
const WATCH_FRAMEBUFFER_BYTES: usize = MonoSurface::buffer_len(board::LCD_WIDTH, board::LCD_HEIGHT);
/// The desk framebuffer is allocated once from PSRAM on the first desk frame.
const DESK_FRAMEBUFFER_BYTES: usize = GraySurface::buffer_len(board::LCD_WIDTH, board::LCD_HEIGHT);

static WATCH_FRAMEBUFFER: StaticCell<[u8; WATCH_FRAMEBUFFER_BYTES]> = StaticCell::new();
static STAGING: StaticCell<[u8; STAGING_BYTES]> = StaticCell::new();

/// A presentation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// The shell rejected rendering the active app.
    Shell,
    /// The surface could not be constructed.
    Surface,
    /// The panel rejected a flush.
    Flush,
}

/// Reusable presentation buffers.
pub struct Workspace {
    /// Watch framebuffer in internal SRAM.
    framebuffer: &'static mut [u8; WATCH_FRAMEBUFFER_BYTES],
    /// Desk framebuffer in PSRAM, allocated on the first desk frame and reused.
    desk: Vec<u8>,
    /// Chunked flush staging.
    staging: &'static mut [u8; STAGING_BYTES],
}

/// Initializes the reusable buffers once per boot.
pub fn workspace() -> Workspace {
    Workspace {
        framebuffer: WATCH_FRAMEBUFFER.init([0; WATCH_FRAMEBUFFER_BYTES]),
        desk: Vec::new(),
        staging: STAGING.init([0; STAGING_BYTES]),
    }
}

/// Renders and flushes the active app for the current mode.
pub fn render<D: Display>(
    display: &mut D,
    shell: &mut Shell<Registry>,
    theme_id: ThemeId,
    workspace: &mut Workspace,
) -> Result<(), RenderError>
where
    D::Error: Debug,
{
    let pixel = RenderPolicy::for_mode(shell.mode()).pixel;
    let theme = Theme::for_id(theme_id, pixel);
    match shell.mode() {
        common::Mode::Watch => render_mono(display, shell, theme, workspace),
        common::Mode::Desk => render_gray(display, shell, theme, workspace),
    }
}

fn render_mono<D: Display>(
    display: &mut D,
    shell: &mut Shell<Registry>,
    theme: &Theme,
    workspace: &mut Workspace,
) -> Result<(), RenderError>
where
    D::Error: Debug,
{
    let data = &mut workspace.framebuffer[..];
    let Ok(mut surface) = MonoSurface::new(board::LCD_WIDTH, board::LCD_HEIGHT, data) else {
        return Err(RenderError::Surface);
    };
    let damage = shell
        .render_active(theme, &mut surface)
        .map_err(|_| RenderError::Shell)?;
    if damage.is_empty() {
        return Ok(());
    }
    flush(display, &surface, &damage, workspace.staging).map_err(|_| RenderError::Flush)
}

fn render_gray<D: Display>(
    display: &mut D,
    shell: &mut Shell<Registry>,
    theme: &Theme,
    workspace: &mut Workspace,
) -> Result<(), RenderError>
where
    D::Error: Debug,
{
    let Workspace { desk, staging, .. } = workspace;
    if desk.len() < DESK_FRAMEBUFFER_BYTES {
        let missing = DESK_FRAMEBUFFER_BYTES - desk.len();
        if desk.try_reserve_exact(missing).is_err() {
            return Err(RenderError::Surface);
        }
        desk.resize(DESK_FRAMEBUFFER_BYTES, 0);
    }
    let Ok(mut surface) = GraySurface::new(board::LCD_WIDTH, board::LCD_HEIGHT, desk) else {
        return Err(RenderError::Surface);
    };
    let damage = shell
        .render_active(theme, &mut surface)
        .map_err(|_| RenderError::Shell)?;
    if damage.is_empty() {
        return Ok(());
    }
    flush(display, &surface, &damage, &mut staging[..]).map_err(|_| RenderError::Flush)
}
