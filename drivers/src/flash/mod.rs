//! Flash partition driver.
//!
//! Owns the single `esp-storage` flash handle, reads the partition table once,
//! and exposes labelled partitions as `NorFlash` regions. Storage policy and
//! TicKV live in the `storage` crate; this module only moves bytes.

mod device;
mod error;
mod region;

pub use device::FlashDevice;
pub use error::FlashError;
pub use region::PartitionFlash;
