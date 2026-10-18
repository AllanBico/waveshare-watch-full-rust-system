//! Settings persistence hooks for the preview session.

use core::fmt::Debug;

use common::Mode;
use esp_println::println;
use hal::Display;
use settings::SettingsStore;

use super::Session;
use crate::log_boot::STORAGE_WRITE_MIN_MV;
use crate::persist::SettingsBackend;

impl Session {
    pub(super) fn apply_brightness<D: Display>(&mut self, display: &mut D)
    where
        D::Error: Debug,
    {
        let percent = self.preview.settings().brightness(self.mode).value();
        let level = (u16::from(percent) * 255 / 100) as u8;
        if self.applied_brightness == Some(level) {
            return;
        }
        match display.set_brightness(level) {
            Ok(()) => self.applied_brightness = Some(level),
            Err(error) => {
                if self.mode == Mode::Desk {
                    println!("[lcd] brightness failed: {:?}", error);
                }
            }
        }
    }
    pub(super) fn sync_settings(&mut self, store: &mut SettingsStore<SettingsBackend>) {
        while let Some((id, value)) = self.preview.pop_setting() {
            if let Err(error) = store.set(id, value) {
                if self.mode == Mode::Desk {
                    println!("[set] rejected {}: {}", id.label(), error);
                }
            }
        }
    }

    pub(super) fn commit(&self, store: &mut SettingsStore<SettingsBackend>, battery_mv: u16) {
        if battery_mv < STORAGE_WRITE_MIN_MV || !store.is_dirty() {
            return;
        }
        match store.flush() {
            Ok(count) if self.mode == Mode::Desk => println!("[set] wrote {} keys", count),
            Ok(_) => {}
            Err(error) => {
                if self.mode == Mode::Desk {
                    println!("[set] flush failed: {}", error);
                }
            }
        }
    }
}
