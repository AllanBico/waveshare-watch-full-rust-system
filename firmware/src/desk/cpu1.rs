//! Desk audio core lifecycle.
//!
//! CPU1 runs a single interrupt-mode executor. The core is started on the
//! first desk entry, parked on every return to watch, and unparked on later
//! desk entries. The guard is retained so parking is always possible.

use esp_hal::interrupt::software::SoftwareInterrupt;
use esp_hal::interrupt::Priority;
use esp_hal::peripherals::CPU_CTRL;
use esp_hal::system::{Cpu, CpuControl, Stack};
use esp_rtos::embassy::InterruptExecutor;
use static_cell::StaticCell;

use common::Cpu1State;

use super::audio::{Clip, Transport};
use super::lab;

const STACK_SIZE: usize = 24 * 1024;

static EXECUTOR: StaticCell<InterruptExecutor<2>> = StaticCell::new();
static APP_CORE_STACK: StaticCell<Stack<STACK_SIZE>> = StaticCell::new();

/// CPU1 and its audio executor.
pub struct Core1 {
    control: CpuControl<'static>,
    guard: Option<esp_hal::system::AppCoreGuard<'static>>,
    executor: Option<&'static mut InterruptExecutor<2>>,
    started: bool,
}

impl Core1 {
    /// Creates the core controller without touching hardware.
    pub fn new(
        cpu_control: CPU_CTRL<'static>,
        software_interrupt: SoftwareInterrupt<'static, 2>,
    ) -> Self {
        let executor = EXECUTOR.init(InterruptExecutor::new(software_interrupt));
        Self {
            control: CpuControl::new(cpu_control),
            guard: None,
            executor: Some(executor),
            started: false,
        }
    }

    /// Starts CPU1 and the audio worker, or unparks it if it already ran.
    pub fn start_audio(&mut self, transport: Transport, clip: Clip) -> bool {
        if self.started {
            return self.resume();
        }

        crate::metrics::set_cpu1_state(Cpu1State::Starting);
        let Some(executor) = self.executor.take() else {
            crate::metrics::set_cpu1_state(Cpu1State::Error);
            return false;
        };
        let stack = APP_CORE_STACK.init(Stack::new());
        let result = self.control.start_app_core(stack, move || {
            let spawner = executor.start(Priority::Priority3);
            if let Ok(token) = lab::task(transport, clip) {
                spawner.spawn(token);
            }
            crate::metrics::set_cpu1_state(Cpu1State::Idle);
            loop {
                // SAFETY: waiti only pauses the core until the next interrupt.
                unsafe { core::arch::asm!("waiti 0") };
            }
        });

        match result {
            Ok(guard) => {
                self.guard = Some(guard);
                self.started = true;
                true
            }
            Err(_) => {
                crate::metrics::set_cpu1_state(Cpu1State::Error);
                false
            }
        }
    }

    /// Unparks CPU1 after a desk re-entry. Audio must already be initialized.
    pub fn resume(&mut self) -> bool {
        if !self.started || self.guard.is_none() {
            crate::metrics::set_cpu1_state(Cpu1State::Error);
            return false;
        }
        self.control.unpark_core(Cpu::AppCpu);
        crate::metrics::set_cpu1_state(Cpu1State::Idle);
        true
    }

    /// Parks CPU1. Audio must already be stopped.
    pub fn park(&mut self) {
        crate::metrics::set_cpu1_state(Cpu1State::Stopping);
        if self.started {
            // SAFETY: CPU0 is parking the other core, never itself.
            unsafe { self.control.park_core(Cpu::AppCpu) };
        }
        crate::metrics::set_cpu1_state(Cpu1State::Off);
    }
}
