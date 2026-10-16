//! Firmware entry point.
//!
//! Phase 1.2: bring up the shared I2C bus and the AXP2101, then drive the
//! power manager through a sleep/awake transition. Mode dispatch
//! (`watch_init` / `desk_init`) and real deep sleep arrive in later phases.

#![no_std]
#![no_main]

extern crate alloc;

use core::cell::RefCell;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::efuse::Efuse;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

use common::{Mode, WakeReason};
use drivers::axp2101::Axp2101;
use drivers::board;
use embedded_hal_bus::i2c::RefCellDevice;
use power::PowerManager;

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

    let mut power = power_bringup(peripherals.I2C0, peripherals.GPIO15, peripherals.GPIO14).await;

    match power.enter_sleep() {
        Ok(()) => println!("[pm] applied watch sleep rail plan"),
        Err(error) => println!("[pm] sleep plan failed: {}", error.to_common()),
    }
    Timer::after(Duration::from_secs(3)).await;

    match power.request_awake(WakeReason::Timer, 10_000, 0) {
        Ok(()) => println!("[pm] awake, state {:?}", power.state()),
        Err(error) => println!("[pm] awake failed: {}", error.to_common()),
    }

    let start = Instant::now();
    loop {
        Timer::after(HEARTBEAT_PERIOD).await;
        let uptime_ms = (Instant::now() - start).as_millis();
        println!("[hb] uptime {} ms", uptime_ms);
    }
}

/// Sets up the shared I2C bus and probes the AXP2101, returning a power manager.
async fn power_bringup(
    i2c: esp_hal::peripherals::I2C0<'static>,
    sda: esp_hal::peripherals::GPIO15<'static>,
    scl: esp_hal::peripherals::GPIO14<'static>,
) -> PowerManager<Axp2101<RefCellDevice<'static, I2c<'static, esp_hal::Blocking>>>> {
    let config = I2cConfig::default().with_frequency(Rate::from_khz(board::I2C_FREQ_HZ / 1000));
    let bus = match I2c::new(i2c, config) {
        Ok(bus) => bus.with_sda(sda).with_scl(scl),
        Err(error) => {
            println!("[pm] i2c init failed: {:?}", error);
            halt().await
        }
    };

    // Leak the bus so the device handle can be `'static`; it lives for the
    // whole program.
    let bus: &'static RefCell<I2c<'static, esp_hal::Blocking>> =
        alloc::boxed::Box::leak(alloc::boxed::Box::new(RefCell::new(bus)));
    let device = RefCellDevice::new(bus);

    let mut pmic = match Axp2101::new(device, board::PMIC_I2C_ADDR) {
        Ok(pmic) => pmic,
        Err(error) => {
            println!("[pm] AXP2101 probe failed: {}", error);
            halt().await
        }
    };

    println!("[pm] AXP2101 id 0x{:02X}", pmic.read_chip_id().unwrap_or(0));
    PowerManager::new(pmic, Mode::Watch)
}

/// Idles forever after a fatal bring-up error.
async fn halt() -> ! {
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}

/// Prints the silicon facts read at runtime, plus the PSRAM size.
fn report_boot(psram_bytes: usize) {
    let revision = Efuse::chip_revision();
    let mac = Efuse::mac_address();

    println!("=== wv2 firmware (Phase 1.2 power bring-up) ===");
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
