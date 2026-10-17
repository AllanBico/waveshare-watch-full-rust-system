//! Audio capture codec contract.
//!
//! Controls the microphone ADC. PCM transport stays above this trait; the
//! codec driver only owns its own registers.

/// A microphone audio codec.
pub trait AudioCapture {
    /// Driver-specific error.
    type Error;

    /// Initializes the codec for 16 kHz, 16-bit I2S capture.
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Sets the microphone gain in dB, in 3 dB steps from 0 to 30.
    fn set_gain(&mut self, db: u8) -> Result<(), Self::Error>;

    /// Powers the codec down.
    fn shutdown(&mut self) -> Result<(), Self::Error>;
}
