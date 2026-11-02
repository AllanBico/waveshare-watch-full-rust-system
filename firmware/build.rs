//! Build-time WiFi credentials.
//!
//! Reads `firmware/wifi.env` or the `WV2_WIFI_SSID` / `WV2_WIFI_PASSWORD`
//! environment variables. The file is gitignored; missing values compile and
//! leave the radio unconfigured.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wifi.env");
    println!("cargo:rerun-if-env-changed=WV2_WIFI_SSID");
    println!("cargo:rerun-if-env-changed=WV2_WIFI_PASSWORD");

    let mut ssid = env::var("WV2_WIFI_SSID").unwrap_or_default();
    let mut password = env::var("WV2_WIFI_PASSWORD").unwrap_or_default();

    if let Ok(manifest) = env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest).join("wifi.env");
        if let Ok(contents) = fs::read_to_string(path) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                match key.trim() {
                    "WV2_WIFI_SSID" => ssid = value.trim().to_string(),
                    "WV2_WIFI_PASSWORD" => password = value.trim().to_string(),
                    _ => {}
                }
            }
        }
    }

    println!("cargo:rustc-env=WV2_WIFI_SSID={ssid}");
    println!("cargo:rustc-env=WV2_WIFI_PASSWORD={password}");
}
