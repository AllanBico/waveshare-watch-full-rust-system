//! Runtime CPU clock selection for the ESP32-S3.
//!
//! esp-hal 1.0 freezes the clock configuration at init. Desk mode needs to
//! move between the watch-safe 80 MHz and the desk 240 MHz without rebooting,
//! so this driver mirrors the S3 register sequence used by esp-hal's own clock
//! driver. APB stays at 80 MHz; only the CPU divider changes.

use core::sync::atomic::{AtomicU16, Ordering};

use esp_hal::peripherals::SYSTEM;

/// A supported CPU clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuSpeed {
    /// Watch-safe CPU clock.
    Mhz80,
    /// Desk CPU clock.
    Mhz240,
}

impl CpuSpeed {
    /// The clock in MHz.
    pub const fn mhz(self) -> u16 {
        match self {
            Self::Mhz80 => 80,
            Self::Mhz240 => 240,
        }
    }

    const fn select(self) -> u8 {
        match self {
            Self::Mhz80 => 0,
            Self::Mhz240 => 2,
        }
    }
}

static CURRENT_MHZ: AtomicU16 = AtomicU16::new(80);

/// The active CPU clock in MHz.
pub fn current_mhz() -> u16 {
    CURRENT_MHZ.load(Ordering::Relaxed)
}

/// Selects the CPU clock.
///
/// The caller is responsible for quiescing the second core and any peripheral
/// that depends on the transition before changing the clock.
pub fn set_speed(speed: CpuSpeed) {
    if current_mhz() == speed.mhz() {
        return;
    }

    // SAFETY: the register writes mirror esp-hal's ESP32-S3 clock driver. They
    // only touch the CPU divider after the caller has quiesced dependent work.
    unsafe {
        SYSTEM::regs()
            .sysclk_conf()
            .modify(|_, w| w.soc_clk_sel().bits(1));
        SYSTEM::regs().cpu_per_conf().modify(|_, w| {
            w.pll_freq_sel().set_bit();
            w.cpuperiod_sel().bits(speed.select())
        });
    }

    esp_hal::rom::ets_update_cpu_frequency_rom(u32::from(speed.mhz()));
    CURRENT_MHZ.store(speed.mhz(), Ordering::Relaxed);
}
