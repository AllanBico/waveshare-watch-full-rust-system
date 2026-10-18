//! Host tests for the settings store.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use common::{Percent, ThemeId};
use hal::Storage;

use crate::{
    DeviceName, Error, SettingId, SettingValue, Settings, SettingsEvent, SettingsStore, TimeFormat,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TestError;

#[derive(Default)]
struct Inner {
    values: BTreeMap<Vec<u8>, Vec<u8>>,
    fail_writes: bool,
}

#[derive(Clone, Default)]
struct TestStorage {
    inner: Rc<RefCell<Inner>>,
}

impl TestStorage {
    fn with_failed_writes() -> Self {
        let storage = Self::default();
        storage.inner.borrow_mut().fail_writes = true;
        storage
    }

    fn put(&self, key: &[u8], value: &[u8]) {
        self.inner
            .borrow_mut()
            .values
            .insert(key.to_vec(), value.to_vec());
    }
}

impl Storage for TestStorage {
    type Error = TestError;

    fn get(&mut self, key: &[u8], value: &mut [u8]) -> Result<usize, Self::Error> {
        let inner = self.inner.borrow();
        let stored = inner.values.get(key).ok_or(TestError)?;
        if value.len() < stored.len() {
            return Err(TestError);
        }
        value[..stored.len()].copy_from_slice(stored);
        Ok(stored.len())
    }

    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error> {
        let mut inner = self.inner.borrow_mut();
        if inner.fail_writes {
            return Err(TestError);
        }
        inner.values.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> Result<bool, Self::Error> {
        Ok(self.inner.borrow_mut().values.remove(key).is_some())
    }
}

#[test]
fn empty_store_loads_defaults() {
    let (store, report) = SettingsStore::load(TestStorage::default());
    assert_eq!(report.loaded, 0);
    assert_eq!(report.missing, crate::SETTING_COUNT as u16);
    assert_eq!(store.settings(), &Settings::new());
    assert!(!store.is_dirty());
}

#[test]
fn set_flush_and_reload_roundtrips() {
    let backend = TestStorage::default();
    {
        let (mut store, _) = SettingsStore::load(backend.clone());
        assert_eq!(
            store.set(
                SettingId::DisplayTheme,
                SettingValue::Theme(ThemeId::HighContrast)
            ),
            Ok(SettingsEvent::Theme)
        );
        assert_eq!(
            store.set(
                SettingId::DisplayTimeFormat,
                SettingValue::TimeFormat(TimeFormat::Hour12)
            ),
            Ok(SettingsEvent::Display)
        );
        assert_eq!(
            store.set(
                SettingId::SoundVolume,
                SettingValue::Percent(Percent::new(25))
            ),
            Ok(SettingsEvent::Sound)
        );
        assert_eq!(store.flush(), Ok(3));
        assert!(!store.is_dirty());
    }

    let (store, report) = SettingsStore::load(backend);
    assert_eq!(report.loaded, 3);
    assert_eq!(report.repaired, 0);
    assert_eq!(store.settings().display.theme, ThemeId::HighContrast);
    assert_eq!(store.settings().display.time_format, TimeFormat::Hour12);
    assert_eq!(store.settings().sound.volume.value(), 25);
}

#[test]
fn corrupt_values_fall_back_to_defaults() {
    let backend = TestStorage::default();
    backend.put(SettingId::DisplayTheme.key(), &[9]);
    backend.put(SettingId::SystemDeviceName.key(), &[3, 0xff]);

    let (store, report) = SettingsStore::load(backend);
    assert_eq!(report.repaired, 2);
    assert_eq!(store.settings().display.theme, ThemeId::Default);
    assert_eq!(store.settings().system.device_name, DeviceName::new());
}

#[test]
fn invalid_values_are_rejected_before_marking_dirty() {
    let (mut store, _) = SettingsStore::load(TestStorage::default());
    assert_eq!(
        store.set(SettingId::DisplayScreenTimeout, SettingValue::Seconds(1)),
        Err(Error::OutOfRange)
    );
    assert!(!store.is_dirty());
}

#[test]
fn failed_writes_stay_dirty() {
    let (mut store, _) = SettingsStore::load(TestStorage::with_failed_writes());
    store
        .set(
            SettingId::DisplayTheme,
            SettingValue::Theme(ThemeId::HighContrast),
        )
        .unwrap();
    assert_eq!(store.flush(), Err(Error::Storage));
    assert_eq!(store.dirty_count(), 1);
}

#[test]
fn unavailable_store_keeps_defaults_and_refuses_writes() {
    let mut store = SettingsStore::<TestStorage>::unavailable();
    assert!(!store.has_backend());
    store
        .set(
            SettingId::DisplayTheme,
            SettingValue::Theme(ThemeId::HighContrast),
        )
        .unwrap();
    assert_eq!(store.flush(), Err(Error::Unavailable));
    assert!(store.is_dirty());
}

#[test]
fn reset_restores_one_key() {
    let backend = TestStorage::default();
    let (mut store, _) = SettingsStore::load(backend.clone());
    store
        .set(
            SettingId::DisplayTheme,
            SettingValue::Theme(ThemeId::HighContrast),
        )
        .unwrap();
    store.flush().unwrap();
    store.reset(SettingId::DisplayTheme).unwrap();
    assert_eq!(store.settings().display.theme, ThemeId::Default);
    assert_eq!(store.flush(), Ok(1));

    let (reloaded, _) = SettingsStore::load(backend);
    assert_eq!(reloaded.settings().display.theme, ThemeId::Default);
}

#[test]
fn change_mask_tracks_keys() {
    let (mut store, _) = SettingsStore::load(TestStorage::default());
    store
        .set(SettingId::ConnectivityWifi, SettingValue::Bool(false))
        .unwrap();
    let mask = store.take_changed();
    assert_eq!(mask, SettingId::ConnectivityWifi.bit());
    assert_eq!(store.take_changed(), 0);
}

#[test]
fn device_name_setting_roundtrips() {
    let backend = TestStorage::default();
    let name = DeviceName::from_bytes(b"studio").unwrap();
    {
        let (mut store, _) = SettingsStore::load(backend.clone());
        store
            .set(SettingId::SystemDeviceName, SettingValue::DeviceName(name))
            .unwrap();
        store.flush().unwrap();
    }
    let (store, _) = SettingsStore::load(backend);
    assert_eq!(store.settings().system.device_name, name);
}
