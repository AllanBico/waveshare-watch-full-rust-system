//! Radio status snapshots shared by firmware and apps.

use core::net::Ipv4Addr;

/// The current WiFi connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiStatus {
    /// No credentials were compiled in.
    Unconfigured,
    /// No connection attempt is active.
    Off,
    /// A connection attempt is in flight.
    Connecting,
    /// The station connected and holds an address.
    Connected(Ipv4Addr),
    /// The station connected but DHCP did not provide an address.
    NoIp,
    /// The attempt failed with a stable error code.
    Failed(u16),
}

impl WifiStatus {
    /// Whether a connection is active or in progress.
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Connecting | Self::Connected(_) | Self::NoIp)
    }
}

/// The current BLE state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BleStatus {
    /// Advertising is off.
    Off,
    /// The watch is discoverable and waiting for a central.
    Advertising,
    /// A central is connected.
    Connected,
    /// The controller failed with a stable error code.
    Failed(u16),
}

impl BleStatus {
    /// Whether advertising or a connection is active.
    pub const fn is_active(self) -> bool {
        matches!(self, Self::Advertising | Self::Connected)
    }
}
