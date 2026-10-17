//! Key-value storage abstraction.
//!
//! The watch reads and writes small byte values by key. The store owns the
//! flash; callers own the buffers. No allocation happens in this contract.

/// A small persistent key-value store.
///
/// Keys are opaque byte strings. Implementations may hash them, so callers
/// must never rely on key ordering or enumeration.
pub trait Storage {
    /// Implementation-specific error.
    type Error;

    /// Reads a value into `value`, returning its length.
    ///
    /// Returns an error if the key is missing or `value` is too small.
    fn get(&mut self, key: &[u8], value: &mut [u8]) -> Result<usize, Self::Error>;

    /// Writes or replaces a value.
    fn set(&mut self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    /// Deletes a key, returning whether it existed.
    fn delete(&mut self, key: &[u8]) -> Result<bool, Self::Error>;
}
