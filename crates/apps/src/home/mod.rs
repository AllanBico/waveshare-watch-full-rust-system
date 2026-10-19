//! The pinned Home app for watch and desk modes.

mod app;
mod desk;
mod state;
mod watch;

pub use app::{Home, RenderError};
pub use state::HomeState;

use common::Mode;

use crate::view::{BuildError, Builder};

/// Home background node.
pub(crate) const BG_ID: u16 = 1;
/// Home content root node.
pub(crate) const ROOT_ID: u16 = 2;
/// Header row node.
pub(crate) const HEADER_ID: u16 = 3;
/// Date label.
pub(crate) const DATE_ID: u16 = 4;
/// Battery label.
pub(crate) const BATTERY_ID: u16 = 5;
/// Header spacer.
pub(crate) const HEADER_SPACER_ID: u16 = 6;
/// Clock label.
pub(crate) const CLOCK_ID: u16 = 7;
/// Top flexible spacer.
pub(crate) const TOP_SPACER_ID: u16 = 8;
/// Bottom flexible spacer.
pub(crate) const BOTTOM_SPACER_ID: u16 = 9;
/// Crown container.
pub(crate) const CROWN_ID: u16 = 10;
/// Content container below the crown.
pub(crate) const CONTENT_ID: u16 = 11;
/// Desk dashboard grid.
pub(crate) const GRID_ID: u16 = 20;
/// Desk alarm tile.
pub(crate) const ALARM_ID: u16 = 21;
/// Desk settings tile.
pub(crate) const SETTINGS_ID: u16 = 22;
/// Desk battery tile.
pub(crate) const DESK_BATTERY_ID: u16 = 23;
/// Desk sync tile.
pub(crate) const SYNC_ID: u16 = 24;
/// Ringing alarm label.
pub(crate) const ALARM_RING_ID: u16 = 25;

/// Builds Home for the active mode.
pub(crate) fn build<'a>(
    state: &'a HomeState,
    mode: Mode,
    view: &mut Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    match mode {
        Mode::Watch => watch::build(state, view),
        Mode::Desk => desk::build(state, view),
    }
}
