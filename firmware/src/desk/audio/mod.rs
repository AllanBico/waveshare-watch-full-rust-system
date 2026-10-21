//! Desk-mode audio codecs and the I2S transport.
//!
//! The codecs are I2S slaves; the ESP32-S3 drives MCLK, BCLK, and LRCK. TX
//! keeps the shared clocks alive while the codecs initialize. The transport is
//! moved into the audio task; the codec handles stay with the session so gain
//! and volume changes are simple I2C writes.

mod analysis;
mod transport;

pub use transport::{Clip, Peripherals, Transport, TransportError};

use drivers::es7210::{DiagnosticRegisters, Es7210, Es7210Error};
use drivers::es8311::{Es8311, Es8311Error};
use embedded_hal::i2c::I2c;
use hal::{Audio as _, AudioCapture as _};

/// Errors from audio bring-up and codec control.
#[allow(dead_code)]
#[derive(Debug)]
pub enum AudioError<E> {
    /// The playback codec rejected an operation.
    Codec(Es8311Error<E>),
    /// The capture codec rejected an operation.
    Capture(Es7210Error<E>),
    /// The I2S transport rejected an operation.
    Transport(TransportError),
    /// The codec has not been initialized.
    NotReady,
}

impl<E> From<Es8311Error<E>> for AudioError<E> {
    fn from(error: Es8311Error<E>) -> Self {
        Self::Codec(error)
    }
}

impl<E> From<Es7210Error<E>> for AudioError<E> {
    fn from(error: Es7210Error<E>) -> Self {
        Self::Capture(error)
    }
}

impl<E> From<TransportError> for AudioError<E> {
    fn from(error: TransportError) -> Self {
        Self::Transport(error)
    }
}

/// The two codecs on the shared I2C bus.
pub struct Codecs<I> {
    codec: Es8311<I>,
    mic: Es7210<I>,
    ready: bool,
}

/// A transport and its codecs, ready for use.
pub struct BroughtUp<I> {
    /// I2S endpoint owned by the audio task.
    pub transport: Transport,
    /// Codec handles owned by the session.
    pub codecs: Codecs<I>,
}

/// Builds the transport and initializes both codecs with clocks running.
pub fn bring_up<I: I2c>(
    peripherals: Peripherals<'static>,
    codec: Es8311<I>,
    mic: Es7210<I>,
) -> Result<BroughtUp<I>, AudioError<I::Error>> {
    let mut transport = Transport::new(peripherals)?;
    let mut codecs = Codecs {
        codec,
        mic,
        ready: false,
    };
    let result = transport.with_silence(|| codecs.init())?;
    result?;
    Ok(BroughtUp { transport, codecs })
}

impl<I: I2c> Codecs<I> {
    /// Initializes playback, then capture.
    pub fn init(&mut self) -> Result<(), AudioError<I::Error>> {
        self.codec.init()?;
        self.mic.init()?;
        self.ready = true;
        Ok(())
    }

    /// Reads the ES7210 key registers for a post-init diagnostic dump.
    pub fn read_mic_diagnostics(&mut self) -> Result<DiagnosticRegisters, Es7210Error<I::Error>> {
        self.mic.read_diagnostics()
    }

    /// Sets playback volume as a percentage.
    pub fn set_volume(&mut self, percent: u8) -> Result<(), AudioError<I::Error>> {
        if !self.ready {
            return Err(AudioError::NotReady);
        }
        self.codec.set_volume(percent).map_err(AudioError::Codec)
    }

    /// Sets microphone gain in 3 dB steps.
    pub fn set_gain(&mut self, db: u8) -> Result<(), AudioError<I::Error>> {
        if !self.ready {
            return Err(AudioError::NotReady);
        }
        self.mic.set_gain(db).map_err(AudioError::Capture)
    }

    /// Mutes or unmutes playback.
    pub fn set_mute(&mut self, muted: bool) -> Result<(), AudioError<I::Error>> {
        if !self.ready {
            return Err(AudioError::NotReady);
        }
        if muted {
            self.codec.mute().map_err(AudioError::Codec)
        } else {
            self.codec.unmute().map_err(AudioError::Codec)
        }
    }

    /// Powers both codecs down.
    pub fn shutdown(&mut self) -> Result<(), AudioError<I::Error>> {
        self.ready = false;
        let mic = self.mic.shutdown().map_err(AudioError::Capture);
        let codec = self.codec.shutdown().map_err(AudioError::Codec);
        mic?;
        codec
    }
}
