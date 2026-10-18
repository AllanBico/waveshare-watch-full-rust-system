//! Loading, mutating, and flushing the settings snapshot.

use hal::Storage;

use crate::value::{SettingValue, MAX_ENCODED_VALUE_LEN};
use crate::{Error, SettingId, Settings, SettingsEvent};

/// Result of reading settings at boot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct LoadReport {
    /// Keys read and applied.
    pub loaded: u16,
    /// Keys that were absent or unreadable.
    pub missing: u16,
    /// Keys that failed validation and kept their default.
    pub repaired: u16,
}

/// Owns the in-memory settings and mirrors dirty keys to a backend.
pub struct SettingsStore<B> {
    backend: Option<B>,
    settings: Settings,
    dirty: u32,
    changed: u32,
}

impl<B: Storage> SettingsStore<B> {
    /// Loads every known setting, falling back to defaults per key.
    pub fn load(backend: B) -> (Self, LoadReport) {
        let mut store = Self {
            backend: Some(backend),
            settings: Settings::new(),
            dirty: 0,
            changed: 0,
        };
        let report = store.load_all();
        (store, report)
    }

    /// Creates a store with defaults and no writable backend.
    pub const fn unavailable() -> Self {
        Self {
            backend: None,
            settings: Settings::new(),
            dirty: 0,
            changed: 0,
        }
    }

    /// The current settings.
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Whether this store can write.
    pub const fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    /// Applies and validates one setting, marking it dirty.
    pub fn set(&mut self, id: SettingId, value: SettingValue) -> Result<SettingsEvent, Error> {
        id.apply(&mut self.settings, value)?;
        self.dirty |= id.bit();
        self.changed |= id.bit();
        Ok(id.event())
    }

    /// Restores one setting to its default.
    pub fn reset(&mut self, id: SettingId) -> Result<SettingsEvent, Error> {
        self.set(id, id.default_value())
    }

    /// Restores every setting to its default.
    pub fn reset_all(&mut self) {
        self.settings = Settings::new();
        self.dirty = u32::MAX;
        self.changed = u32::MAX;
    }

    /// Number of settings waiting to be written.
    pub const fn dirty_count(&self) -> u32 {
        self.dirty.count_ones()
    }

    /// Whether any setting is waiting to be written.
    pub const fn is_dirty(&self) -> bool {
        self.dirty != 0
    }

    /// Takes the mask of settings changed since the last call.
    pub fn take_changed(&mut self) -> u32 {
        core::mem::take(&mut self.changed)
    }

    /// Discards pending writes. Use only when writes are unsafe.
    pub fn discard(&mut self) {
        self.dirty = 0;
    }

    /// Writes every dirty setting, returning how many were stored.
    ///
    /// Failed keys stay dirty so a later flush can retry them.
    pub fn flush(&mut self) -> Result<usize, Error> {
        if self.dirty == 0 {
            return Ok(0);
        }
        let Some(backend) = self.backend.as_mut() else {
            return Err(Error::Unavailable);
        };
        let settings = &self.settings;
        let mut dirty = self.dirty;
        let mut written = 0usize;
        let mut first_error = None;
        let mut buffer = [0u8; MAX_ENCODED_VALUE_LEN];

        for id in SettingId::ALL {
            let bit = id.bit();
            if dirty & bit == 0 {
                continue;
            }
            let result = id.read(settings).encode(&mut buffer).and_then(|len| {
                backend
                    .set(id.key(), &buffer[..len])
                    .map_err(|_| Error::Storage)
            });
            match result {
                Ok(()) => {
                    dirty &= !bit;
                    written += 1;
                }
                Err(error) => {
                    if first_error.is_none() {
                        first_error = Some(error);
                    }
                }
            }
        }
        self.dirty = dirty;
        match first_error {
            Some(error) => Err(error),
            None => Ok(written),
        }
    }

    fn load_all(&mut self) -> LoadReport {
        let Some(backend) = self.backend.as_mut() else {
            return LoadReport::default();
        };
        let mut report = LoadReport::default();
        let mut buffer = [0u8; MAX_ENCODED_VALUE_LEN];
        for id in SettingId::ALL {
            match backend.get(id.key(), &mut buffer) {
                Ok(len) => match id.decode(&buffer[..len]) {
                    Ok(value) => match id.apply(&mut self.settings, value) {
                        Ok(()) => report.loaded = report.loaded.saturating_add(1),
                        Err(_) => report.repaired = report.repaired.saturating_add(1),
                    },
                    Err(_) => report.repaired = report.repaired.saturating_add(1),
                },
                Err(_) => report.missing = report.missing.saturating_add(1),
            }
        }
        report
    }
}
