//! Stable key hashing for TicKV.
//!
//! TicKV stores only a 64-bit hash. Changing this algorithm would orphan every
//! existing key, so the constants and the finalizer are frozen with the format.

/// FNV-1a offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
/// SplitMix64 increment.
const SPLITMIX_INCREMENT: u64 = 0x9e37_79b9_7f4a_7c15;
/// SplitMix64 first multiplier.
const SPLITMIX_MULTIPLIER_1: u64 = 0xbf58_476d_1ce4_e5b9;
/// SplitMix64 second multiplier.
const SPLITMIX_MULTIPLIER_2: u64 = 0x94d0_49bb_1331_11eb;

/// Hashes a key into the 64-bit space TicKV expects.
///
/// FNV-1a gives a cheap base; the SplitMix64 finalizer spreads sequential keys
/// across TicKV regions. The result is never `0` or `u64::MAX`, both of which
/// TicKV rejects.
pub const fn hash_key(key: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET;
    let mut index = 0;
    while index < key.len() {
        hash ^= key[index] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        index += 1;
    }

    hash = hash.wrapping_add(SPLITMIX_INCREMENT);
    hash = (hash ^ (hash >> 30)).wrapping_mul(SPLITMIX_MULTIPLIER_1);
    hash = (hash ^ (hash >> 27)).wrapping_mul(SPLITMIX_MULTIPLIER_2);
    hash ^= hash >> 31;

    match hash {
        0 => 1,
        u64::MAX => u64::MAX - 1,
        other => other,
    }
}

/// The hash of TicKV's super key, used to detect an initialized partition.
pub const MAIN_KEY_HASH: u64 = hash_key(b"tickv-super-key");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_vector_is_stable() {
        assert_eq!(MAIN_KEY_HASH, 0x36fe_fd72_2241_7c7a);
    }

    #[test]
    fn hash_matches_the_tickv_super_key() {
        assert_eq!(b"tickv-super-key", tickv::MAIN_KEY);
    }

    #[test]
    fn hash_never_returns_tickv_rejected_values() {
        for key in [b"".as_slice(), b"a", b"settings/brightness", b"log/0001"] {
            let hash = hash_key(key);
            assert_ne!(hash, 0);
            assert_ne!(hash, u64::MAX);
        }
    }

    #[test]
    fn sequential_keys_spread() {
        let first = hash_key(b"log/0000");
        let second = hash_key(b"log/0001");
        assert_ne!(first, second);
        assert_ne!(first & 0xffff, second & 0xffff);
    }
}
