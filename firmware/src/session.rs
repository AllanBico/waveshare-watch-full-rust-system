//! Shell session and awake loops.
//!
//! Owns shell boot, first-light presentation, and the per-mode awake loop.
//! Rendering is still the firmware presenter; the app view seam arrives in 7.1.

use core::fmt::Debug;

use embassy_time::{Duration, Instant, Timer};
use esp_println::println;
use hal::{Display, Power};
use power::PowerManager;

use common::{Mode, Percent, Region, ThemeId, Timestamp};
use drivers::board;
use shell::{Shell, ShellError};

use crate::boot::VBUS_POLL_PERIOD;
use crate::bringup;
use crate::registry::{Registry, HOME_ID};

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

/// Presents the label and idles for the watch awake budget.
pub async fn run_watch<D: Display>(
    display: &mut D,
    shell: &Shell<Registry>,
    theme: ThemeId,
    staging: &mut [u8],
    awake_ms: u32,
) where
    D::Error: Debug,
{
    bringup::draw(display, shell.mode(), theme, staging);
    Timer::after(Duration::from_millis(u64::from(awake_ms))).await;
}

/// Presents the label and ticks the shell until USB is removed.
pub async fn run_desk<D: Display, P: Power>(
    display: &mut D,
    shell: &mut Shell<Registry>,
    theme: ThemeId,
    staging: &mut [u8],
    power: &mut PowerManager<P>,
) where
    D::Error: Debug,
{
    bringup::draw(display, shell.mode(), theme, staging);
    let started = Instant::now();
    while power.is_vbus_present().unwrap_or(true) {
        Timer::after(VBUS_POLL_PERIOD).await;
        if let Err(error) = shell.tick((Instant::now() - started).as_millis()) {
            println!("[shell] tick failed: {:?}", error);
        }
    }
}
