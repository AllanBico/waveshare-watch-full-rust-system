//! Desk system metrics: executor load, SRAM, PSRAM, and CPU clock.
//!
//! The load figure is the share of wall time the embassy executor spent
//! polling tasks. Interrupt time is excluded, which keeps the sample cheap and
//! deterministic on the desk loop.

use core::sync::atomic::{AtomicBool, AtomicU16, AtomicU32, AtomicU8, Ordering};

use common::{Cpu1State, Percent, SystemStats};
use embassy_time::Instant;
use esp_alloc::MemoryCapability;
use esp_rtos::embassy::Callbacks;

static TIMING_READY: AtomicBool = AtomicBool::new(false);
static POLL_START_MS: AtomicU32 = AtomicU32::new(0);
static BUSY_MS: AtomicU32 = AtomicU32::new(0);
static CPU1_BUSY_US: AtomicU32 = AtomicU32::new(0);
static CPU1_STATE: AtomicU8 = AtomicU8::new(Cpu1State::Off as u8);
static PLAY_IN_PEAK: AtomicU16 = AtomicU16::new(0);
static PLAY_OUT_PEAK: AtomicU16 = AtomicU16::new(0);

/// Enables load timing once the scheduler timer is running.
pub fn enable() {
    TIMING_READY.store(true, Ordering::Relaxed);
}

/// Records the CPU1 worker state for the desk sampler.
pub fn set_cpu1_state(state: Cpu1State) {
    CPU1_STATE.store(state as u8, Ordering::Relaxed);
}

/// Adds time spent polling the CPU1 worker.
pub fn add_cpu1_busy_us(micros: u32) {
    CPU1_BUSY_US.fetch_add(micros, Ordering::Relaxed);
}

/// Records the first-buffer playback peaks before and after filtering.
pub fn set_play_peaks(input: u16, output: u16) {
    PLAY_IN_PEAK.store(input, Ordering::Relaxed);
    PLAY_OUT_PEAK.store(output, Ordering::Relaxed);
}

/// The last recorded playback peaks.
pub fn play_peaks() -> (u16, u16) {
    (
        PLAY_IN_PEAK.load(Ordering::Relaxed),
        PLAY_OUT_PEAK.load(Ordering::Relaxed),
    )
}

fn cpu1_state() -> Cpu1State {
    match CPU1_STATE.load(Ordering::Relaxed) {
        x if x == Cpu1State::Starting as u8 => Cpu1State::Starting,
        x if x == Cpu1State::Idle as u8 => Cpu1State::Idle,
        x if x == Cpu1State::Audio as u8 => Cpu1State::Audio,
        x if x == Cpu1State::Stopping as u8 => Cpu1State::Stopping,
        x if x == Cpu1State::Error as u8 => Cpu1State::Error,
        _ => Cpu1State::Off,
    }
}

/// Executor callbacks that accumulate polled time for [`Sampler`].
pub struct LoadCallbacks;

impl Callbacks for LoadCallbacks {
    fn before_poll(&mut self) {
        if !TIMING_READY.load(Ordering::Relaxed) {
            return;
        }
        let now = Instant::now().as_millis() as u32;
        let previous = POLL_START_MS.swap(now, Ordering::Relaxed);
        if previous != 0 {
            BUSY_MS.fetch_add(now.wrapping_sub(previous), Ordering::Relaxed);
        }
    }

    fn on_idle(&mut self) {
        if !TIMING_READY.load(Ordering::Relaxed) {
            return;
        }
        let now = Instant::now().as_millis() as u32;
        let previous = POLL_START_MS.swap(0, Ordering::Relaxed);
        if previous != 0 {
            BUSY_MS.fetch_add(now.wrapping_sub(previous), Ordering::Relaxed);
        }
    }
}

/// A one-second system stats sampler.
pub struct Sampler {
    last_busy_ms: u32,
    last_cpu1_busy_us: u32,
    last_wall_ms: u32,
}

impl Sampler {
    /// Creates a sampler primed at the current instant.
    pub fn new() -> Self {
        Self {
            last_busy_ms: BUSY_MS.load(Ordering::Relaxed),
            last_cpu1_busy_us: CPU1_BUSY_US.load(Ordering::Relaxed),
            last_wall_ms: Instant::now().as_millis() as u32,
        }
    }

    /// Samples the metrics accumulated since the previous call.
    pub fn sample(&mut self) -> SystemStats {
        let busy = BUSY_MS.load(Ordering::Relaxed);
        let cpu1_busy = CPU1_BUSY_US.load(Ordering::Relaxed);
        let now = Instant::now().as_millis() as u32;
        let work = busy.wrapping_sub(self.last_busy_ms);
        let cpu1_work_us = cpu1_busy.wrapping_sub(self.last_cpu1_busy_us);
        let wall = now.wrapping_sub(self.last_wall_ms);
        self.last_busy_ms = busy;
        self.last_cpu1_busy_us = cpu1_busy;
        self.last_wall_ms = now;
        let cpu = if wall == 0 {
            0
        } else {
            (u64::from(work) * 100 / u64::from(wall)).min(100)
        };
        let cpu1 = if wall == 0 {
            0
        } else {
            (u64::from(cpu1_work_us) * 100 / (u64::from(wall) * 1_000)).min(100)
        };
        let state = cpu1_state();
        SystemStats {
            cpu_percent: Percent::new(cpu as u8),
            cpu1_percent: Percent::new(if state == Cpu1State::Off {
                0
            } else {
                cpu1 as u8
            }),
            cpu1_state: state,
            cpu_mhz: clock_mhz(),
            ..memory()
        }
    }
}

impl Default for Sampler {
    fn default() -> Self {
        Self::new()
    }
}

/// A snapshot without a load window, for the first desk frame.
pub fn snapshot() -> SystemStats {
    SystemStats {
        cpu_percent: Percent::new(0),
        cpu1_percent: Percent::new(0),
        cpu1_state: cpu1_state(),
        cpu_mhz: clock_mhz(),
        ..memory()
    }
}

fn clock_mhz() -> u16 {
    drivers::clock::current_mhz()
}

fn memory() -> SystemStats {
    let (ram_used_kb, ram_total_kb) = internal_ram_kb();
    let (psram_used_kb, psram_total_kb) = psram_kb();
    SystemStats {
        cpu_percent: Percent::new(0),
        cpu1_percent: Percent::new(0),
        cpu1_state: Cpu1State::Off,
        cpu_mhz: 0,
        ram_used_kb,
        ram_total_kb,
        psram_used_kb,
        psram_total_kb,
    }
}

fn psram_kb() -> (u32, u32) {
    let stats = esp_alloc::HEAP.stats();
    let mut used = 0usize;
    let mut total = 0usize;
    for region in stats.region_stats.iter().flatten() {
        if region.capabilities.contains(MemoryCapability::External) {
            used = used.saturating_add(region.used);
            total = total.saturating_add(region.size);
        }
    }
    (to_kb(used), to_kb(total))
}

fn internal_ram_kb() -> (u32, u32) {
    unsafe extern "C" {
        static _data_start: u32;
        static _stack_end_cpu0: u32;
        static _stack_start_cpu0: u32;
    }

    // SAFETY: these linker symbols mark the static-data and stack bounds of
    // the RWDATA region. They are read-only addresses, not dereferenced.
    let data_start = core::ptr::addr_of!(_data_start) as usize;
    let stack_end = core::ptr::addr_of!(_stack_end_cpu0) as usize;
    let stack_start = core::ptr::addr_of!(_stack_start_cpu0) as usize;
    let stack_pointer = esp_hal::xtensa_lx::get_stack_pointer() as usize;
    let static_used = stack_end.saturating_sub(data_start);
    let stack_used = stack_start.saturating_sub(stack_pointer);
    let used = static_used.saturating_add(stack_used);
    let total = stack_start.saturating_sub(data_start);
    (to_kb(used), to_kb(total))
}

fn to_kb(bytes: usize) -> u32 {
    u32::try_from(bytes / 1024).unwrap_or(u32::MAX)
}
