//! WiFi station task with DHCP.

use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::{Runner, StackResources};
use embassy_sync08::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync08::channel::Receiver;
use esp_hal::peripherals::WIFI;
use esp_hal::rng::Rng;
use esp_println::println;
use esp_radio::wifi::{sta::StationConfig, Config, ControllerConfig, Interface, WifiController};
use static_cell::StaticCell;

use common::WifiStatus;

use super::{WifiCommand, WIFI_STATUS};

static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();

const SSID: &str = match option_env!("WV2_WIFI_SSID") {
    Some(value) => value,
    None => "",
};
const PASSWORD: &str = match option_env!("WV2_WIFI_PASSWORD") {
    Some(value) => value,
    None => "",
};

/// Whether a compile-time SSID exists.
pub fn has_credentials() -> bool {
    !SSID.is_empty()
}

/// Runs the WiFi controller until the device leaves desk mode.
#[embassy_executor::task]
pub(super) async fn task(
    wifi: WIFI<'static>,
    commands: Receiver<'static, CriticalSectionRawMutex, WifiCommand, 4>,
    spawner: Spawner,
) {
    if !has_credentials() {
        report(WifiStatus::Unconfigured);
        return;
    }

    let station = Config::Station(
        StationConfig::default()
            .with_ssid(SSID)
            .with_password(PASSWORD.into()),
    );
    let (mut controller, interfaces) = match esp_radio::wifi::new(
        wifi,
        ControllerConfig::default().with_initial_config(station),
    ) {
        Ok(parts) => parts,
        Err(error) => {
            println!("[wifi] init failed: {:?}", error);
            report(WifiStatus::Failed(1));
            return;
        }
    };

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;
    let (stack, runner) = embassy_net::new(
        interfaces.station,
        embassy_net::Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );
    if let Ok(token) = net_task(runner) {
        spawner.spawn(token);
    }
    println!("[wifi] ready");

    loop {
        match commands.receive().await {
            WifiCommand::Stop => report(WifiStatus::Off),
            WifiCommand::Connect => run_session(&mut controller, stack, &commands).await,
        }
    }
}

async fn run_session(
    controller: &mut WifiController<'static>,
    stack: embassy_net::Stack<'static>,
    commands: &Receiver<'static, CriticalSectionRawMutex, WifiCommand, 4>,
) {
    report(WifiStatus::Connecting);
    match select(controller.connect_async(), wait_stop(commands)).await {
        Either::First(Ok(_)) => {}
        Either::First(Err(_)) => {
            report(WifiStatus::Failed(2));
            return;
        }
        Either::Second(()) => {
            let _ = controller.disconnect_async().await;
            report(WifiStatus::Off);
            return;
        }
    }

    match select(stack.wait_config_up(), wait_stop(commands)).await {
        Either::First(_) => {
            let status = match stack.config_v4() {
                Some(config) => WifiStatus::Connected(config.address.address()),
                None => WifiStatus::NoIp,
            };
            report(status);
        }
        Either::Second(()) => {
            let _ = controller.disconnect_async().await;
            report(WifiStatus::Off);
            return;
        }
    }

    match select(controller.wait_for_disconnect_async(), wait_stop(commands)).await {
        Either::First(_) => report(WifiStatus::Off),
        Either::Second(()) => {
            let _ = controller.disconnect_async().await;
            report(WifiStatus::Off);
        }
    }
}

async fn wait_stop(commands: &Receiver<'static, CriticalSectionRawMutex, WifiCommand, 4>) {
    loop {
        if matches!(commands.receive().await, WifiCommand::Stop) {
            return;
        }
    }
}

fn report(status: WifiStatus) {
    WIFI_STATUS.sender().send(status);
    println!("[wifi] {:?}", status);
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, Interface<'static>>) {
    runner.run().await
}
