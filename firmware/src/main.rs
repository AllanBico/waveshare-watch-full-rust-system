//! Firmware entry point.
//!
//! Phase 0.2: boot, report silicon facts over USB serial, and run an embassy
//! heartbeat so the async runtime and timers are proven without busy-waiting.
//! Mode dispatch (`watch_init` / `desk_init`) arrives once VBUS detection is
//! available in Phase 1.

#![no_std]
#![no_main]

extern crate alloc;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::efuse::Efuse;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

use esp_alloc as _;
use esp_backtrace as _;

esp_bootloader_esp_idf::esp_app_desc!();

/// CPU clock for bring-up. Mode-based clock switching comes later.
const BRINGUP_CPU_CLOCK_MHZ: u32 = 160;

/// Heartbeat period.
const HEARTBEAT_PERIOD: Duration = Duration::from_secs(1);

/// SRAM heap size reserved for the runtime.
const SRAM_HEAP_BYTES: usize = 64 * 1024;

#[esp_rtos::main]
async fn main(_spawner: Spawner) {
    esp_alloc::heap_allocator!(size: SRAM_HEAP_BYTES);

    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::_160MHz));

    let (_, psram_bytes) = esp_hal::psram::psram_raw_parts(&peripherals.PSRAM);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0);

    report_boot(psram_bytes);

    let start = Instant::now();
    loop {
        Timer::after(HEARTBEAT_PERIOD).await;
        let uptime_ms = (Instant::now() - start).as_millis();
        println!("[hb] uptime {} ms", uptime_ms);
    }
}

/// Prints the silicon facts read at runtime, plus the PSRAM size.
fn report_boot(psram_bytes: usize) {
    let revision = Efuse::chip_revision();
    let mac = Efuse::mac_address();

    println!("=== wv2 firmware (Phase 0.2 bring-up) ===");
    println!(
        "chip:  {} rev v{}.{}",
        esp_hal::chip!(),
        revision / 100,
        revision % 100
    );
    println!("cpu:   {} MHz", BRINGUP_CPU_CLOCK_MHZ);
    println!(
        "mac:   {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    println!("psram: {} bytes", psram_bytes);
    match esp_hal::system::reset_reason() {
        Some(reason) => println!("reset: {:?}", reason),
        None => println!("reset: unknown"),
    }
}
