//! Audio codec contract.
//!
//! Controls the playback codec. PCM transport and the capture codec live
//! above this trait so a driver only owns its own registers.

/// A playback audio codec.
pub trait Audio {
    /// Driver-specific error.
    type Error;

    /// Initializes the codec for 16 kHz, 16-bit I2S playback.
    fn init(&mut self) -> Result<(), Self::Error>;

    /// Sets the output volume as a percentage, `0` to `100`.
    ///
    /// The mapping never exceeds 0 dB so the amplifier input cannot clip.
    fn set_volume(&mut self, percent: u8) -> Result<(), Self::Error>;

    /// Mutes the codec output.
    fn mute(&mut self) -> Result<(), Self::Error>;

    /// Unmutes the codec output.
    fn unmute(&mut self) -> Result<(), Self::Error>;

    /// Powers the codec down to its lowest register state.
    fn shutdown(&mut self) -> Result<(), Self::Error>;
}
