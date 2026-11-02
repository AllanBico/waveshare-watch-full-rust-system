//! BLE peripheral task: advertise, accept one connection, expose a status read.

use embassy_futures::join::join;
use embassy_futures::select::select;
use embassy_sync08::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync08::channel::Receiver;
use embassy_time::{Duration, Timer};
use esp_hal::efuse::{interface_mac_address, InterfaceMacAddress};
use esp_hal::peripherals::BT;
use esp_println::println;
use esp_radio::ble::controller::BleConnector;
use static_cell::StaticCell;
use trouble_host::prelude::*;

use common::BleStatus;

use super::{BleCommand, BLE_STATUS};

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 2;
const NAME: &str = "WV2";

static RESOURCES: StaticCell<
    HostResources<DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, 1>,
> = StaticCell::new();

#[gatt_server]
struct Server {
    connectivity: ConnectivityService,
}

#[gatt_service(uuid = "a1b2c3d4-5e6f-4a7b-8c9d-0e1f2a3b4c5d")]
struct ConnectivityService {
    #[characteristic(uuid = "a1b2c3d4-5e6f-4a7b-8c9d-0e1f2a3b4c5e", read, value = 1u8)]
    status: u8,
}

/// Runs the BLE controller until the device leaves desk mode.
#[embassy_executor::task]
pub(super) async fn task(
    bt: BT<'static>,
    commands: Receiver<'static, CriticalSectionRawMutex, BleCommand, 4>,
) {
    let connector = match BleConnector::new(bt, Default::default()) {
        Ok(connector) => connector,
        Err(error) => {
            println!("[ble] init failed: {:?}", error);
            report(BleStatus::Failed(1));
            return;
        }
    };
    let controller: ExternalController<_, 1> = ExternalController::new(connector);

    let mac = interface_mac_address(InterfaceMacAddress::Bluetooth);
    let mut address = [0u8; 6];
    address.copy_from_slice(mac.as_bytes());
    let address = Address::random(address);

    let resources = RESOURCES.init(HostResources::new());
    let stack = trouble_host::new(controller, resources).set_random_address(address);
    let Host {
        mut peripheral,
        mut runner,
        ..
    } = stack.build();

    let server = match Server::new_with_config(GapConfig::Peripheral(PeripheralConfig {
        name: NAME,
        appearance: &appearance::power_device::GENERIC_POWER_DEVICE,
    })) {
        Ok(server) => server,
        Err(_) => {
            report(BleStatus::Failed(2));
            return;
        }
    };

    let _ = join(
        runner.run(),
        control_loop(&mut peripheral, &server, &commands),
    )
    .await;
}

async fn control_loop<C: Controller>(
    peripheral: &mut Peripheral<'_, C, DefaultPacketPool>,
    server: &Server<'_>,
    commands: &Receiver<'static, CriticalSectionRawMutex, BleCommand, 4>,
) {
    loop {
        match commands.receive().await {
            BleCommand::Stop => report(BleStatus::Off),
            BleCommand::Start => {
                select(advertise_loop(peripheral, server), wait_stop(commands)).await;
                report(BleStatus::Off);
            }
        }
    }
}

async fn advertise_loop<C: Controller>(
    peripheral: &mut Peripheral<'_, C, DefaultPacketPool>,
    server: &Server<'_>,
) {
    let mut adv_data = [0; 31];
    let adv_len = match AdStructure::encode_slice(
        &[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::CompleteLocalName(NAME.as_bytes()),
        ],
        &mut adv_data[..],
    ) {
        Ok(len) => len,
        Err(_) => {
            report(BleStatus::Failed(3));
            return;
        }
    };
    let params = AdvertisementParameters {
        interval_min: Duration::from_millis(100),
        interval_max: Duration::from_millis(100),
        ..Default::default()
    };

    loop {
        report(BleStatus::Advertising);
        match peripheral
            .advertise(
                &params,
                Advertisement::ConnectableScannableUndirected {
                    adv_data: &adv_data[..adv_len],
                    scan_data: &[],
                },
            )
            .await
        {
            Ok(advertiser) => match advertiser.accept().await {
                Ok(accepted) => match accepted.with_attribute_server(&server.server) {
                    Ok(connection) => {
                        report(BleStatus::Connected);
                        gatt_events(server, &connection).await;
                    }
                    Err(_) => report(BleStatus::Failed(4)),
                },
                Err(_) => report(BleStatus::Failed(5)),
            },
            Err(_) => {
                report(BleStatus::Failed(6));
                Timer::after(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn wait_stop(commands: &Receiver<'static, CriticalSectionRawMutex, BleCommand, 4>) {
    loop {
        if matches!(commands.receive().await, BleCommand::Stop) {
            return;
        }
    }
}

async fn gatt_events(server: &Server<'_>, connection: &GattConnection<'_, '_, DefaultPacketPool>) {
    let status = server.connectivity.status;
    loop {
        match connection.next().await {
            GattConnectionEvent::Disconnected { .. } => return,
            GattConnectionEvent::Gatt { event } => {
                if let GattEvent::Read(read) = &event {
                    if read.handle() == status.handle {
                        let _ = server.get(&status);
                    }
                }
                if let Ok(reply) = event.accept() {
                    reply.send().await;
                }
            }
            _ => {}
        }
    }
}

fn report(status: BleStatus) {
    BLE_STATUS.sender().send(status);
    println!("[ble] {:?}", status);
}
