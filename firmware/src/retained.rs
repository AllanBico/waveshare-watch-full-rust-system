//! Retained ring storage in RTC slow memory.
//!
//! The ring survives deep sleep. The linker section is not initialized on
//! reset, so `Ring::new` validates its magic before use.

use logging::{Retained, Ring};

#[esp_hal::ram(unstable(rtc_slow, persistent))]
static mut RETAINED: Retained = [0; logging::RETAINED_BYTES];

/// Runs `f` with the retained ring.
pub fn with_ring<R>(f: impl FnOnce(&mut Ring<'_>) -> R) -> R {
    // SAFETY: only the main task touches the ring, and interrupts never do.
    // The static is a plain byte array, valid for its full length.
    let bytes = unsafe { &mut *core::ptr::addr_of_mut!(RETAINED) };
    f(&mut Ring::new(bytes))
}
