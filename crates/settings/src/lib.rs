//! Global runtime settings.
//!
//! Owns the typed settings schema, defaults, validation, dirty tracking, and
//! per-key persistence over a [`hal::Storage`] backend. Settings are loaded
//! once at boot and mutated through [`SettingsStore`]; the rest of the system
//! reads a snapshot and reacts to [`SettingsEvent`]s.
//!
//! Compile-time configuration (pins, addresses, safety limits) does not
//! belong here. Neither does device state such as logs, alarms, or sync
//! checkpoints.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

mod codec;
mod encoding;
mod error;
mod id;
mod keys;
mod settings;
mod store;
#[cfg(test)]
mod tests;
mod value;

pub use error::Error;
pub use id::{SettingGroup, SettingId, SettingsEvent, SETTING_COUNT};
pub use settings::{
    ConnectivitySettings, DisplaySettings, PowerSettings, Settings, SoundSettings, SystemSettings,
};
pub use store::{LoadReport, SettingsStore};
pub use value::{
    DateFormat, DeviceName, FirstDay, SettingValue, TimeFormat, Units, ValueKind,
    MAX_DEVICE_NAME_LEN, MAX_ENCODED_VALUE_LEN,
};
