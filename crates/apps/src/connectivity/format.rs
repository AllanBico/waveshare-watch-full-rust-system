//! Fixed-buffer text for WiFi and BLE status rows.

use common::{BleStatus, WifiStatus};

/// A rendered row subtitle and action label.
pub(super) struct RowText {
    /// Secondary text.
    pub subtitle: [u8; 56],
    /// Subtitle length.
    pub subtitle_len: usize,
    /// Trailing action label.
    pub action: &'static str,
}

/// Formats the WiFi row.
pub(super) fn wifi(status: WifiStatus) -> RowText {
    let mut row = RowText {
        subtitle: [0; 56],
        subtitle_len: 0,
        action: if status.is_active() {
            "Stop"
        } else {
            "Connect"
        },
    };
    match status {
        WifiStatus::Unconfigured => row.put(b"No credentials configured"),
        WifiStatus::Off => row.put(b"Not connected"),
        WifiStatus::Connecting => row.put(b"Connecting..."),
        WifiStatus::Connected(ip) => {
            row.put(b"Connected  ");
            row.ip(ip);
        }
        WifiStatus::NoIp => row.put(b"Connected, no IP"),
        WifiStatus::Failed(code) => {
            row.put(b"Failed E");
            row.number(code);
        }
    }
    row
}

/// Formats the BLE row.
pub(super) fn ble(status: BleStatus) -> RowText {
    let mut row = RowText {
        subtitle: [0; 56],
        subtitle_len: 0,
        action: if status.is_active() { "Stop" } else { "Start" },
    };
    match status {
        BleStatus::Off => row.put(b"Not advertising"),
        BleStatus::Advertising => row.put(b"Discoverable as WV2"),
        BleStatus::Connected => row.put(b"Connected to a device"),
        BleStatus::Failed(code) => {
            row.put(b"Failed E");
            row.number(code);
        }
    }
    row
}

impl RowText {
    /// The subtitle as text.
    pub(super) fn text(&self) -> &str {
        core::str::from_utf8(&self.subtitle[..self.subtitle_len]).unwrap_or("")
    }

    fn put(&mut self, bytes: &[u8]) {
        for byte in bytes {
            if self.subtitle_len < self.subtitle.len() {
                self.subtitle[self.subtitle_len] = *byte;
                self.subtitle_len += 1;
            }
        }
    }

    fn number(&mut self, value: u16) {
        let mut digits = [0u8; 5];
        let mut len = 0usize;
        let mut value = value;
        if value == 0 {
            digits[0] = b'0';
            len = 1;
        } else {
            while value > 0 {
                digits[len] = b'0' + (value % 10) as u8;
                value /= 10;
                len += 1;
            }
        }
        while len > 0 {
            len -= 1;
            self.put(&[digits[len]]);
        }
    }

    fn ip(&mut self, ip: core::net::Ipv4Addr) {
        let octets = ip.octets();
        for (index, octet) in octets.iter().enumerate() {
            if index > 0 {
                self.put(b".");
            }
            self.number(u16::from(*octet));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connected_wifi_shows_its_ip() {
        let row = wifi(WifiStatus::Connected(core::net::Ipv4Addr::new(
            192, 168, 1, 5,
        )));
        assert_eq!(row.text(), "Connected  192.168.1.5");
    }

    #[test]
    fn missing_credentials_are_visible() {
        let row = wifi(WifiStatus::Unconfigured);
        assert_eq!(row.text(), "No credentials configured");
    }
}
