//! Desk system metrics shared with the UI.

use crate::units::Percent;

/// Runtime state of the desk audio core.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Cpu1State {
    /// Parked, or never started on this boot.
    #[default]
    Off,
    /// Starting the core and audio worker.
    Starting,
    /// Running, waiting for audio work.
    Idle,
    /// Running an audio capture or playback operation.
    Audio,
    /// Draining audio before the core is parked.
    Stopping,
    /// Bring-up or audio failed.
    Error,
}

/// A snapshot of the desk system metrics shown on Home.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemStats {
    /// Share of wall time CPU0 spent polling the executor.
    pub cpu_percent: Percent,
    /// Share of wall time CPU1 spent polling the audio worker.
    pub cpu1_percent: Percent,
    /// Current CPU1 state.
    pub cpu1_state: Cpu1State,
    /// Current CPU clock in MHz.
    pub cpu_mhz: u16,
    /// Internal SRAM currently used, in KiB.
    pub ram_used_kb: u32,
    /// Internal SRAM available to the firmware, in KiB.
    pub ram_total_kb: u32,
    /// PSRAM currently used, in KiB.
    pub psram_used_kb: u32,
    /// PSRAM available to the allocator, in KiB.
    pub psram_total_kb: u32,
}

impl SystemStats {
    /// An empty snapshot used before the first sample.
    pub const fn empty() -> Self {
        Self {
            cpu_percent: Percent::new(0),
            cpu1_percent: Percent::new(0),
            cpu1_state: Cpu1State::Off,
            cpu_mhz: 0,
            ram_used_kb: 0,
            ram_total_kb: 0,
            psram_used_kb: 0,
            psram_total_kb: 0,
        }
    }
}

impl Default for SystemStats {
    fn default() -> Self {
        Self::empty()
    }
}
