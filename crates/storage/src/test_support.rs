//! In-memory NOR flash for host tests.

use std::cell::RefCell;
use std::rc::Rc;

use embedded_storage::nor_flash::{
    ErrorType, MultiwriteNorFlash, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use embedded_storage::ReadStorage;

const WORD_SIZE: u32 = 4;
const SECTOR_SIZE: u32 = 4096;

/// Errors injected by the mock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockError {
    /// Access outside the flash.
    OutOfBounds,
    /// Misaligned write, read, or erase.
    NotAligned,
    /// Injected failure.
    Injected,
}

impl NorFlashError for MockError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            MockError::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            MockError::NotAligned => NorFlashErrorKind::NotAligned,
            MockError::Injected => NorFlashErrorKind::Other,
        }
    }
}

struct State {
    data: Vec<u8>,
    erase_count: usize,
    write_count: usize,
    fail_writes: bool,
}

/// Shared handle for inspecting a mock flash after it moves into storage.
#[derive(Clone)]
pub struct MockHandle {
    state: Rc<RefCell<State>>,
}

impl MockHandle {
    /// Current flash contents.
    pub fn data(&self) -> Vec<u8> {
        self.state.borrow().data.clone()
    }

    /// Number of sector erases so far.
    pub fn erase_count(&self) -> usize {
        self.state.borrow().erase_count
    }

    /// Number of word writes so far.
    pub fn write_count(&self) -> usize {
        self.state.borrow().write_count
    }

    /// Writes a byte directly, bypassing flash semantics.
    pub fn corrupt(&self, offset: usize, value: u8) {
        self.state.borrow_mut().data[offset] = value;
    }

    /// Makes subsequent writes fail.
    pub fn fail_writes(&self, fail: bool) {
        self.state.borrow_mut().fail_writes = fail;
    }
}

/// In-memory NOR flash with sector erase.
pub struct MockFlash {
    state: Rc<RefCell<State>>,
}

impl MockFlash {
    /// Creates an erased flash of `size` bytes.
    pub fn new(size: usize) -> Self {
        Self {
            state: Rc::new(RefCell::new(State {
                data: vec![0xFF; size],
                erase_count: 0,
                write_count: 0,
                fail_writes: false,
            })),
        }
    }

    /// Creates another view over an existing flash.
    pub fn from_handle(handle: MockHandle) -> Self {
        Self {
            state: handle.state,
        }
    }

    /// Shared inspection handle.
    pub fn handle(&self) -> MockHandle {
        MockHandle {
            state: Rc::clone(&self.state),
        }
    }
}

impl ErrorType for MockFlash {
    type Error = MockError;
}

impl ReadStorage for MockFlash {
    type Error = MockError;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        let state = self.state.borrow();
        let start = offset as usize;
        let end = start + bytes.len();
        let source = state.data.get(start..end).ok_or(MockError::OutOfBounds)?;
        bytes.copy_from_slice(source);
        Ok(())
    }

    fn capacity(&self) -> usize {
        self.state.borrow().data.len()
    }
}

impl ReadNorFlash for MockFlash {
    const READ_SIZE: usize = WORD_SIZE as usize;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if !offset.is_multiple_of(WORD_SIZE) || !bytes.len().is_multiple_of(WORD_SIZE as usize) {
            return Err(MockError::NotAligned);
        }
        ReadStorage::read(self, offset, bytes)
    }

    fn capacity(&self) -> usize {
        self.state.borrow().data.len()
    }
}

impl NorFlash for MockFlash {
    const WRITE_SIZE: usize = WORD_SIZE as usize;
    const ERASE_SIZE: usize = SECTOR_SIZE as usize;

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if !offset.is_multiple_of(WORD_SIZE) || !bytes.len().is_multiple_of(WORD_SIZE as usize) {
            return Err(MockError::NotAligned);
        }
        let mut state = self.state.borrow_mut();
        if state.fail_writes {
            return Err(MockError::Injected);
        }
        let start = offset as usize;
        let end = start + bytes.len();
        let target = state
            .data
            .get_mut(start..end)
            .ok_or(MockError::OutOfBounds)?;
        target.copy_from_slice(bytes);
        state.write_count += 1;
        Ok(())
    }

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if !from.is_multiple_of(SECTOR_SIZE) || !to.is_multiple_of(SECTOR_SIZE) || from > to {
            return Err(MockError::NotAligned);
        }
        let mut state = self.state.borrow_mut();
        let start = from as usize;
        let end = to as usize;
        let target = state
            .data
            .get_mut(start..end)
            .ok_or(MockError::OutOfBounds)?;
        target.fill(0xFF);
        state.erase_count += target.len() / SECTOR_SIZE as usize;
        Ok(())
    }
}

impl MultiwriteNorFlash for MockFlash {}
