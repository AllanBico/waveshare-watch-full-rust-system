//! Shared register-file I2C mock for host tests.

use embedded_hal::i2c::{Error, ErrorKind, ErrorType, I2c, NoAcknowledgeSource, Operation};

/// Error type for the mock bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockError {
    /// The transfer was rejected.
    Nack,
}

impl Error for MockError {
    fn kind(&self) -> ErrorKind {
        ErrorKind::NoAcknowledge(NoAcknowledgeSource::Unknown)
    }
}

/// A 256-byte register file that implements the I2C trait.
#[derive(Debug)]
pub struct MockI2c {
    regs: [u8; 256],
    fail: bool,
    writes: Vec<(u8, u8)>,
}

impl Default for MockI2c {
    fn default() -> Self {
        Self {
            regs: [0; 256],
            fail: false,
            writes: Vec::new(),
        }
    }
}

impl MockI2c {
    /// Creates an all-zero register file.
    pub fn new() -> Self {
        Self::default()
    }

    /// Presets a register value, for builder-style setup.
    pub fn with_reg(mut self, reg: u8, value: u8) -> Self {
        self.set_reg(reg, value);
        self
    }

    /// Sets a register value.
    pub fn set_reg(&mut self, reg: u8, value: u8) {
        self.regs[reg as usize] = value;
    }

    /// Reads a register value.
    pub fn reg(&self, reg: u8) -> u8 {
        self.regs[reg as usize]
    }

    /// Makes every transfer fail until cleared.
    pub fn set_fail(&mut self, fail: bool) {
        self.fail = fail;
    }

    /// The recorded `(register, value)` writes.
    pub fn writes(&self) -> &[(u8, u8)] {
        &self.writes
    }
}

impl ErrorType for MockI2c {
    type Error = MockError;
}

impl I2c for MockI2c {
    fn read(&mut self, _address: u8, _read: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    fn write(&mut self, _address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        if self.fail {
            return Err(MockError::Nack);
        }
        if let Some((reg, values)) = bytes.split_first() {
            for (offset, value) in values.iter().enumerate() {
                let address = (u16::from(*reg) + offset as u16) as u8;
                self.regs[address as usize] = *value;
                self.writes.push((address, *value));
            }
        }
        Ok(())
    }

    fn write_read(
        &mut self,
        _address: u8,
        write: &[u8],
        read: &mut [u8],
    ) -> Result<(), Self::Error> {
        if self.fail {
            return Err(MockError::Nack);
        }
        let start = write.first().copied().unwrap_or(0) as usize;
        for (offset, byte) in read.iter_mut().enumerate() {
            *byte = self.regs.get(start + offset).copied().unwrap_or(0);
        }
        Ok(())
    }

    fn transaction(
        &mut self,
        address: u8,
        operations: &mut [Operation<'_>],
    ) -> Result<(), Self::Error> {
        for operation in operations {
            match operation {
                Operation::Read(buffer) => self.read(address, buffer)?,
                Operation::Write(buffer) => self.write(address, buffer)?,
            }
        }
        Ok(())
    }
}
