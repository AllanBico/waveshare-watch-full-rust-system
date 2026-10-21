//! Audio Lab contracts shared by the desk app and the firmware service.
//!
//! The app emits [`LabIntent`]s and renders [`LabEvent`]s. The firmware owns
//! the I2S transport, codecs, and clip buffer; none of these types carry
//! hardware state.

/// Capture and playback sample rate.
pub const SAMPLE_RATE_HZ: u32 = 16_000;
/// Bytes in one interleaved 16-bit stereo frame.
pub const FRAME_BYTES: usize = 4;
/// Shortest tuner recording.
pub const MIN_RECORD_MS: u16 = 1_000;
/// Longest tuner recording.
pub const MAX_RECORD_MS: u16 = 10_000;
/// Default tuner recording.
pub const DEFAULT_RECORD_MS: u16 = 5_000;
/// Microphone gain step.
pub const GAIN_STEP_DB: u8 = 3;
/// Highest microphone gain.
pub const MAX_GAIN_DB: u8 = 30;
/// Full-scale magnitude for a 16-bit sample.
pub const FULL_SCALE: u16 = 32_768;
/// Magnitude considered clipped.
pub const CLIP_LEVEL: u16 = 32_700;

/// dBFS thresholds in 3 dB steps, from 0 dB down to -96 dB.
const DBFS_STEPS: [u16; 33] = [
    32_768, 23_198, 16_423, 11_627, 8_231, 5_827, 4_125, 2_920, 2_068, 1_464, 1_036, 734, 519, 368,
    260, 184, 130, 92, 65, 46, 33, 23, 16, 12, 8, 6, 4, 3, 2, 1, 1, 1, 1,
];

/// One channel's level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Meter {
    /// Largest absolute sample.
    pub peak: u16,
    /// Root-mean-square sample magnitude.
    pub rms: u16,
    /// Whether any sample reached [`CLIP_LEVEL`].
    pub clipped: bool,
}

impl Meter {
    /// A silent meter.
    pub const SILENT: Self = Self {
        peak: 0,
        rms: 0,
        clipped: false,
    };

    /// Peak as a percentage of full scale.
    pub const fn peak_percent(self) -> u8 {
        if self.peak >= FULL_SCALE {
            100
        } else {
            (self.peak as u32 * 100 / FULL_SCALE as u32) as u8
        }
    }

    /// Peak in dBFS, rounded to the nearest 3 dB step.
    pub const fn peak_dbfs(self) -> i16 {
        if self.peak == 0 {
            return -96;
        }
        let mut index = 0;
        while index + 1 < DBFS_STEPS.len() {
            if self.peak >= DBFS_STEPS[index + 1] {
                break;
            }
            index += 1;
        }
        -(index as i16 * 3)
    }
}

/// One microphone's wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MicChannel {
    /// MIC1, the left I2S slot.
    One,
    /// MIC2, the right I2S slot.
    Two,
}

impl MicChannel {
    /// The next channel in the cycle.
    pub const fn next(self) -> Self {
        match self {
            Self::One => Self::Two,
            Self::Two => Self::One,
        }
    }

    /// Short label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::One => "MIC1",
            Self::Two => "MIC2",
        }
    }
}

/// A recording or playback request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabCommand {
    /// Capture for a duration. A non-zero `tone_hz` also plays a test tone.
    Record {
        /// Capture length in milliseconds.
        duration_ms: u16,
        /// Test tone frequency, or zero for silence.
        tone_hz: u16,
    },
    /// Stop the current operation.
    Stop,
    /// Play the last clip from one microphone.
    PlayClip {
        /// Channel to play.
        channel: MicChannel,
    },
    /// Play a generated tone.
    PlayTone {
        /// Tone frequency.
        frequency_hz: u16,
        /// Tone length.
        duration_ms: u16,
    },
}

/// A request emitted by the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabIntent {
    /// Start, stop, or play.
    Command(LabCommand),
    /// Set the capture gain in 3 dB steps.
    SetMicGain(u8),
    /// Set playback volume as a percentage.
    SetVolume(u8),
    /// Mute or unmute playback.
    SetMute(bool),
}

/// A tuner failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabError {
    /// The audio hardware has not been initialized.
    NotReady,
    /// Another operation is already running.
    Busy,
    /// The capture clip could not be allocated.
    Alloc,
    /// No clip has been recorded.
    NoClip,
    /// The codec rejected an operation.
    Codec,
    /// The I2S transport rejected an operation.
    I2s,
    /// The DMA transfer failed.
    Dma,
    /// The DMA ran late (overrun or underrun).
    DmaLate,
    /// The DMA buffer was misaligned.
    DmaAlign,
    /// The DMA ran out of descriptors.
    DmaDescriptors,
    /// The DMA rejected a descriptor.
    DmaDescriptor,
    /// The DMA buffer overflowed.
    DmaOverflow,
    /// The DMA buffer was too small.
    DmaTooSmall,
    /// The DMA buffer is not in a supported memory region.
    DmaMemory,
    /// The DMA chunk size is invalid.
    DmaChunk,
    /// Capture returned no frames.
    Empty,
}

impl LabError {
    /// Short label for the app.
    pub const fn label(self) -> &'static str {
        match self {
            Self::NotReady => "audio not ready",
            Self::Busy => "audio busy",
            Self::Alloc => "clip alloc failed",
            Self::NoClip => "no clip yet",
            Self::Codec => "codec error",
            Self::I2s => "i2s error",
            Self::Dma => "dma error",
            Self::DmaLate => "dma late",
            Self::DmaAlign => "dma align",
            Self::DmaDescriptors => "dma descriptors",
            Self::DmaDescriptor => "dma descriptor",
            Self::DmaOverflow => "dma overflow",
            Self::DmaTooSmall => "dma buffer small",
            Self::DmaMemory => "dma memory",
            Self::DmaChunk => "dma chunk",
            Self::Empty => "no samples",
        }
    }
}

/// Result of one capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClipReport {
    /// Frames captured.
    pub frames: u32,
    /// Time from start to stop.
    pub elapsed_ms: u32,
    /// MIC1 level.
    pub mic1: Meter,
    /// MIC2 level.
    pub mic2: Meter,
    /// MIC1 dominant tone, zero when unclear.
    pub mic1_hz: u16,
    /// MIC2 dominant tone, zero when unclear.
    pub mic2_hz: u16,
    /// MIC1 DC offset.
    pub mic1_dc: i16,
    /// MIC2 DC offset.
    pub mic2_dc: i16,
    /// Tone played during capture, zero for silence.
    pub tone_hz: u16,
}

impl ClipReport {
    /// Frames a correct capture should produce for its measured wall time.
    pub const fn expected_frames(self) -> u32 {
        self.elapsed_ms * SAMPLE_RATE_HZ / 1_000
    }

    /// Capture rate as a percentage of the configured rate.
    pub const fn rate_percent(self) -> u16 {
        if self.elapsed_ms == 0 {
            return 0;
        }
        let expected = self.expected_frames() as u64;
        if expected == 0 {
            return 0;
        }
        ((self.frames as u64 * 100) / expected) as u16
    }
}

/// A report from the firmware service.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabEvent {
    /// An operation started.
    Started {
        /// Configured length.
        duration_ms: u16,
        /// Test tone frequency, zero for silence.
        tone_hz: u16,
    },
    /// Live levels while capturing.
    Recording {
        /// Milliseconds since capture started.
        elapsed_ms: u16,
        /// MIC1 level.
        mic1: Meter,
        /// MIC2 level.
        mic2: Meter,
    },
    /// Capture finished.
    Recorded(ClipReport),
    /// Live progress while playing.
    Playback {
        /// Milliseconds since playback started.
        elapsed_ms: u16,
        /// Clip length.
        duration_ms: u16,
    },
    /// Playback finished.
    Finished,
    /// The current operation was stopped.
    Stopped,
    /// The operation failed.
    Error(LabError),
}

#[cfg(test)]
mod tests;
