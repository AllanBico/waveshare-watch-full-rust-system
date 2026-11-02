//! Desk-only WiFi and BLE control.
//!
//! Radios are never initialized on a watch boot. The app queues intents; this
//! module owns the controller tasks and exposes the latest status snapshot.

mod ble;
mod wifi;

use embassy_executor::Spawner;
use embassy_sync08::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync08::channel::Channel;
use embassy_sync08::watch::Watch;
use esp_hal::peripherals::{BT, WIFI};

use common::{BleStatus, WifiStatus};

pub use wifi::has_credentials;

static WIFI_STATUS: Watch<CriticalSectionRawMutex, WifiStatus, 2> =
    Watch::new_with(WifiStatus::Off);
static BLE_STATUS: Watch<CriticalSectionRawMutex, BleStatus, 2> = Watch::new_with(BleStatus::Off);
static WIFI_COMMANDS: Channel<CriticalSectionRawMutex, WifiCommand, 4> = Channel::new();
static BLE_COMMANDS: Channel<CriticalSectionRawMutex, BleCommand, 4> = Channel::new();

/// A request for the WiFi task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiCommand {
    /// Connect to the compiled-in network.
    Connect,
    /// Disconnect and stay idle.
    Stop,
}

/// A request for the BLE task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BleCommand {
    /// Advertise as WV2.
    Start,
    /// Stop advertising and drop any connection.
    Stop,
}

/// Everything the radio tasks need from the boot chain.
pub struct Inputs {
    /// WiFi peripheral.
    pub wifi: WIFI<'static>,
    /// Bluetooth peripheral.
    pub bt: BT<'static>,
    /// Executor used to spawn the controller tasks.
    pub spawner: Spawner,
}

/// The desk radio owner.
pub struct Radio {
    wifi: Option<WIFI<'static>>,
    bt: Option<BT<'static>>,
    spawner: Spawner,
    wifi_started: bool,
    ble_started: bool,
}

impl Radio {
    /// Creates the radio owner without touching hardware.
    pub fn new(inputs: Inputs) -> Self {
        let status = if has_credentials() {
            WifiStatus::Off
        } else {
            WifiStatus::Unconfigured
        };
        WIFI_STATUS.sender().send(status);
        BLE_STATUS.sender().send(BleStatus::Off);
        Self {
            wifi: Some(inputs.wifi),
            bt: Some(inputs.bt),
            spawner: inputs.spawner,
            wifi_started: false,
            ble_started: false,
        }
    }

    /// The latest radio snapshot.
    pub fn status(&self) -> (WifiStatus, BleStatus) {
        (
            WIFI_STATUS.try_get().unwrap_or(WifiStatus::Off),
            BLE_STATUS.try_get().unwrap_or(BleStatus::Off),
        )
    }

    /// Requests a WiFi connection or stop.
    pub fn wifi(&mut self, command: WifiCommand) {
        if matches!(command, WifiCommand::Connect) {
            self.ensure_wifi();
        }
        if self.wifi_started {
            let _ = WIFI_COMMANDS.sender().try_send(command);
        }
    }

    /// Requests BLE advertising or stop.
    pub fn ble(&mut self, command: BleCommand) {
        if matches!(command, BleCommand::Start) {
            self.ensure_ble();
        }
        if self.ble_started {
            let _ = BLE_COMMANDS.sender().try_send(command);
        }
    }

    /// Stops both radios.
    pub fn stop_all(&mut self) {
        if self.wifi_started {
            let _ = WIFI_COMMANDS.sender().try_send(WifiCommand::Stop);
        }
        if self.ble_started {
            let _ = BLE_COMMANDS.sender().try_send(BleCommand::Stop);
        }
    }

    fn ensure_wifi(&mut self) {
        if self.wifi_started {
            return;
        }
        let Some(wifi) = self.wifi.take() else {
            return;
        };
        if let Ok(token) = wifi::task(wifi, WIFI_COMMANDS.receiver(), self.spawner) {
            self.spawner.spawn(token);
        }
        self.wifi_started = true;
    }

    fn ensure_ble(&mut self) {
        if self.ble_started {
            return;
        }
        let Some(bt) = self.bt.take() else {
            return;
        };
        if let Ok(token) = ble::task(bt, BLE_COMMANDS.receiver()) {
            self.spawner.spawn(token);
        }
        self.ble_started = true;
    }
}
