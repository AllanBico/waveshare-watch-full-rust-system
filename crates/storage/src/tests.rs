//! Host tests for the TicKV storage layer.

use super::*;
use crate::test_support::{MockFlash, MockHandle};

const FLASH_SIZE: usize = 8 * REGION_SIZE;

fn leaked_buffer() -> &'static mut [u8; REGION_SIZE] {
    Box::leak(Box::new([0u8; REGION_SIZE]))
}

fn open(
    flash: MockFlash,
    provisioning: Provisioning,
) -> Result<(TicKvStorage<'static, MockFlash>, OpenOutcome), Error> {
    TicKvStorage::open(flash, leaked_buffer(), provisioning)
}

fn provisioned(size: usize) -> (TicKvStorage<'static, MockFlash>, MockHandle) {
    let flash = MockFlash::new(size);
    let handle = flash.handle();
    let (store, outcome) = open(flash, Provisioning::Allowed).unwrap();
    assert_eq!(outcome, OpenOutcome::Provisioned);
    (store, handle)
}

#[test]
fn blank_partition_is_not_erased_when_provisioning_is_forbidden() {
    let flash = MockFlash::new(FLASH_SIZE);
    let handle = flash.handle();
    let result = open(flash, Provisioning::Forbidden);
    assert!(matches!(result, Err(Error::NotReady)));
    assert_eq!(handle.erase_count(), 0);
    assert_eq!(handle.write_count(), 0);
}

#[test]
fn write_reboot_read_persists() {
    let flash = MockFlash::new(FLASH_SIZE);
    let handle = flash.handle();
    let (mut store, outcome) = open(flash, Provisioning::Allowed).unwrap();
    assert_eq!(outcome, OpenOutcome::Provisioned);
    assert!(handle.data().iter().any(|byte| *byte != 0xFF));

    store.set(b"settings/theme", b"desk").unwrap();
    let mut value = [0u8; 16];
    assert_eq!(store.get(b"settings/theme", &mut value).unwrap(), 4);
    assert_eq!(&value[..4], b"desk");

    let erases = handle.erase_count();
    drop(store);

    let flash = MockFlash::from_handle(handle.clone());
    let (mut store, outcome) = open(flash, Provisioning::Forbidden).unwrap();
    assert_eq!(outcome, OpenOutcome::Ready);
    assert_eq!(handle.erase_count(), erases);

    let mut value = [0u8; 16];
    assert_eq!(store.get(b"settings/theme", &mut value).unwrap(), 4);
    assert_eq!(&value[..4], b"desk");
}

#[test]
fn overwrite_replaces_the_value() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    store.set(b"key", b"first").unwrap();
    store.set(b"key", b"second").unwrap();

    let mut value = [0u8; 16];
    assert_eq!(store.get(b"key", &mut value).unwrap(), 6);
    assert_eq!(&value[..6], b"second");
}

#[test]
fn delete_reports_presence() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    store.set(b"key", b"value").unwrap();
    assert!(store.delete(b"key").unwrap());
    assert!(!store.delete(b"key").unwrap());
    assert!(!store.delete(b"never").unwrap());
    assert!(matches!(
        store.get(b"key", &mut [0u8; 8]),
        Err(Error::NotFound)
    ));
}

#[test]
fn missing_key_is_not_found() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    assert!(matches!(
        store.get(b"missing", &mut [0u8; 8]),
        Err(Error::NotFound)
    ));
}

#[test]
fn unaligned_values_round_trip() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    for length in 1..=9usize {
        let key = vec![b'k'; length];
        let value = vec![length as u8; length];
        store.set(&key, &value).unwrap();

        let mut read = vec![0u8; length];
        assert_eq!(store.get(&key, &mut read).unwrap(), length);
        assert_eq!(read, value);
    }
}

#[test]
fn small_buffer_reports_the_needed_length() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    store.set(b"key", b"hello").unwrap();
    assert!(matches!(
        store.get(b"key", &mut [0u8; 2]),
        Err(Error::BufferTooSmall(5))
    ));
}

#[test]
fn oversized_value_is_rejected_before_flash() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    let value = vec![0xA5; MAX_VALUE_LEN + 1];
    assert!(matches!(
        store.set(b"big", &value),
        Err(Error::ValueTooLarge)
    ));
}

#[test]
fn maximum_value_round_trips() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    let value = vec![0x3C; MAX_VALUE_LEN];
    store.set(b"max", &value).unwrap();

    let mut read = vec![0u8; MAX_VALUE_LEN];
    assert_eq!(store.get(b"max", &mut read).unwrap(), MAX_VALUE_LEN);
    assert_eq!(read, value);
}

#[test]
fn invalid_keys_are_rejected() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    assert!(matches!(store.set(b"", b"x"), Err(Error::InvalidKey)));
    let long = vec![b'k'; MAX_KEY_LEN + 1];
    assert!(matches!(store.set(&long, b"x"), Err(Error::InvalidKey)));
}

#[test]
fn invalid_region_is_rejected() {
    let flash = MockFlash::new(REGION_SIZE + 1);
    assert!(matches!(
        open(flash, Provisioning::Allowed),
        Err(Error::InvalidRegion)
    ));
}

#[test]
fn force_reformats_non_blank_flash() {
    let flash = MockFlash::new(FLASH_SIZE);
    let handle = flash.handle();
    handle.corrupt(0, 0x00);
    let (store, outcome) = open(flash, Provisioning::Force).unwrap();
    assert_eq!(outcome, OpenOutcome::Provisioned);
    assert!(handle.erase_count() > 0);
    drop(store);
}

#[test]
fn corrupt_store_is_not_reformatted() {
    let (store, handle) = provisioned(FLASH_SIZE);
    drop(store);

    let regions = FLASH_SIZE / REGION_SIZE;
    let region = (MAIN_KEY_HASH as usize & 0xFFFF) % regions;
    let start = region * REGION_SIZE;
    for offset in 0..32 {
        handle.corrupt(start + offset, 0x00);
    }

    let erases = handle.erase_count();
    let flash = MockFlash::from_handle(handle.clone());
    let result = open(flash, Provisioning::Allowed);
    assert!(matches!(result, Err(Error::Corrupt)));
    assert_eq!(handle.erase_count(), erases);
}

#[test]
fn force_recovers_a_corrupt_super_key() {
    let (store, handle) = provisioned(FLASH_SIZE);
    drop(store);

    let regions = FLASH_SIZE / REGION_SIZE;
    let region = (MAIN_KEY_HASH as usize & 0xFFFF) % regions;
    let start = region * REGION_SIZE;
    for offset in 0..32 {
        handle.corrupt(start + offset, 0x00);
    }

    let erases = handle.erase_count();
    let flash = MockFlash::from_handle(handle.clone());
    let (store, outcome) = open(flash, Provisioning::Force).unwrap();
    assert_eq!(outcome, OpenOutcome::Provisioned);
    assert!(handle.erase_count() > erases);
    drop(store);
}

#[test]
fn schema_mismatch_is_rejected() {
    let (mut store, handle) = provisioned(FLASH_SIZE);
    store.set(b"meta/schema", &2u16.to_le_bytes()).unwrap();
    drop(store);

    let flash = MockFlash::from_handle(handle);
    let result = open(flash, Provisioning::Forbidden);
    assert!(matches!(result, Err(Error::VersionMismatch)));
}

#[test]
fn full_store_reports_full() {
    let (mut store, _) = provisioned(4 * REGION_SIZE);
    let value = vec![0xA5; 1000];

    let mut last = Ok(());
    for index in 0..100u32 {
        last = store.set(&index.to_le_bytes(), &value);
        if last.is_err() {
            break;
        }
    }
    assert!(matches!(last, Err(Error::Full)));
}

#[test]
fn flash_failures_map_to_flash() {
    let (mut store, handle) = provisioned(FLASH_SIZE);
    handle.fail_writes(true);
    assert!(matches!(store.set(b"key", b"value"), Err(Error::Flash)));
}

#[test]
fn gc_is_safe_on_a_clean_store() {
    let (mut store, _) = provisioned(FLASH_SIZE);
    store.set(b"key", b"value").unwrap();
    assert!(store.gc().is_ok());
}

#[test]
fn errors_map_to_common_codes() {
    assert_eq!(Error::Full.to_common(), common::Error::StorageFull);
    assert_eq!(Error::Corrupt.to_common(), common::Error::StorageCorrupt);
    assert_eq!(Error::NotFound.to_common(), common::Error::NotFound);
    assert_eq!(Error::Flash.to_common(), common::Error::HardwareFailure);
    assert_eq!(Error::NotReady.to_common(), common::Error::NotReady);
}
