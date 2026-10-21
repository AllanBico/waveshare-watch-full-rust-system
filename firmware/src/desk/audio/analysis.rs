//! Pure sample analysis and tone generation for the audio tuner.

use voice::lab::{Meter, CLIP_LEVEL, SAMPLE_RATE_HZ};

/// Sine table with 256 points.
const SINE: [i16; 256] = [
    0, 804, 1608, 2410, 3212, 4011, 4808, 5602, 6393, 7179, 7962, 8739, 9512, 10278, 11039, 11793,
    12539, 13279, 14010, 14732, 15446, 16151, 16846, 17530, 18204, 18868, 19519, 20159, 20787,
    21403, 22005, 22594, 23170, 23731, 24279, 24811, 25329, 25832, 26319, 26790, 27245, 27683,
    28105, 28510, 28898, 29268, 29621, 29956, 30273, 30571, 30852, 31113, 31356, 31580, 31785,
    31971, 32137, 32285, 32412, 32521, 32609, 32678, 32728, 32757, 32767, 32757, 32728, 32678,
    32609, 32521, 32412, 32285, 32137, 31971, 31785, 31580, 31356, 31113, 30852, 30571, 30273,
    29956, 29621, 29268, 28898, 28510, 28105, 27683, 27245, 26790, 26319, 25832, 25329, 24811,
    24279, 23731, 23170, 22594, 22005, 21403, 20787, 20159, 19519, 18868, 18204, 17530, 16846,
    16151, 15446, 14732, 14010, 13279, 12539, 11793, 11039, 10278, 9512, 8739, 7962, 7179, 6393,
    5602, 4808, 4011, 3212, 2410, 1608, 804, 0, -804, -1608, -2410, -3212, -4011, -4808, -5602,
    -6393, -7179, -7962, -8739, -9512, -10278, -11039, -11793, -12539, -13279, -14010, -14732,
    -15446, -16151, -16846, -17530, -18204, -18868, -19519, -20159, -20787, -21403, -22005, -22594,
    -23170, -23731, -24279, -24811, -25329, -25832, -26319, -26790, -27245, -27683, -28105, -28510,
    -28898, -29268, -29621, -29956, -30273, -30571, -30852, -31113, -31356, -31580, -31785, -31971,
    -32137, -32285, -32412, -32521, -32609, -32678, -32728, -32757, -32767, -32757, -32728, -32678,
    -32609, -32521, -32412, -32285, -32137, -31971, -31785, -31580, -31356, -31113, -30852, -30571,
    -30273, -29956, -29621, -29268, -28898, -28510, -28105, -27683, -27245, -26790, -26319, -25832,
    -25329, -24811, -24279, -23731, -23170, -22594, -22005, -21403, -20787, -20159, -19519, -18868,
    -18204, -17530, -16846, -16151, -15446, -14732, -14010, -13279, -12539, -11793, -11039, -10278,
    -9512, -8739, -7962, -7179, -6393, -5602, -4808, -4011, -3212, -2410, -1608, -804,
];

/// Running level accumulator for one capture.
pub struct Levels {
    peak1: u16,
    peak2: u16,
    sumsq1: u64,
    sumsq2: u64,
    frames: u32,
    clipped1: bool,
    clipped2: bool,
}

impl Levels {
    /// An empty accumulator.
    pub const fn new() -> Self {
        Self {
            peak1: 0,
            peak2: 0,
            sumsq1: 0,
            sumsq2: 0,
            frames: 0,
            clipped1: false,
            clipped2: false,
        }
    }

    /// Adds one stereo frame.
    pub fn push(&mut self, left: i16, right: i16) {
        let left_mag = left.unsigned_abs();
        let right_mag = right.unsigned_abs();
        self.peak1 = self.peak1.max(left_mag);
        self.peak2 = self.peak2.max(right_mag);
        self.sumsq1 += u64::from(left_mag) * u64::from(left_mag);
        self.sumsq2 += u64::from(right_mag) * u64::from(right_mag);
        self.clipped1 |= left_mag >= CLIP_LEVEL;
        self.clipped2 |= right_mag >= CLIP_LEVEL;
        self.frames = self.frames.saturating_add(1);
    }

    /// Current MIC1 and MIC2 levels.
    pub fn snapshot(&self) -> (Meter, Meter) {
        let frames = self.frames.max(1);
        (
            Meter {
                peak: self.peak1,
                rms: integer_sqrt(self.sumsq1 / u64::from(frames)),
                clipped: self.clipped1,
            },
            Meter {
                peak: self.peak2,
                rms: integer_sqrt(self.sumsq2 / u64::from(frames)),
                clipped: self.clipped2,
            },
        )
    }
}

/// Mean sample for one interleaved channel.
pub fn dc_offset(clip: &[u8], offset: usize) -> i16 {
    let mut sum = 0i64;
    let mut count = 0u32;
    let mut index = offset;
    while index + 1 < clip.len() {
        sum += i64::from(i16::from_le_bytes([clip[index], clip[index + 1]]));
        count += 1;
        index += 4;
    }
    if count == 0 {
        0
    } else {
        (sum / i64::from(count)).clamp(i64::from(i16::MIN), i64::from(i16::MAX)) as i16
    }
}

/// Estimates a dominant tone from one interleaved channel.
pub fn estimate_hz(clip: &[u8], offset: usize) -> u16 {
    let frames = clip.len() / 4;
    if frames < 2 {
        return 0;
    }
    let mut peak = 0u16;
    let mut index = offset;
    while index + 1 < clip.len() {
        peak = peak.max(i16::from_le_bytes([clip[index], clip[index + 1]]).unsigned_abs());
        index += 4;
    }
    if peak < 200 {
        return 0;
    }
    let threshold = i32::from(peak / 4);
    let mut previous = 0i32;
    let mut crossings = 0u32;
    let mut index = offset;
    while index + 1 < clip.len() {
        let sample = i32::from(i16::from_le_bytes([clip[index], clip[index + 1]]));
        if previous < -threshold && sample >= threshold {
            crossings += 1;
        }
        previous = sample;
        index += 4;
    }
    if crossings < 2 {
        return 0;
    }
    let hz = u64::from(crossings) * u64::from(SAMPLE_RATE_HZ) / frames.saturating_sub(1) as u64;
    hz.min(u64::from(u16::MAX)) as u16
}

/// A phase-continuous sine generator.
pub struct Tone {
    phase: u32,
    step: u32,
}

impl Tone {
    /// Creates a tone at `frequency_hz`.
    pub fn new(frequency_hz: u16) -> Self {
        let step = (u64::from(frequency_hz) << 32) / u64::from(SAMPLE_RATE_HZ);
        Self {
            phase: 0,
            step: step as u32,
        }
    }

    /// Fills stereo interleaved frames.
    pub fn fill(&mut self, out: &mut [u8]) {
        for frame in out.chunks_exact_mut(4) {
            let sample = SINE[(self.phase >> 24) as usize] / 4;
            let bytes = sample.to_le_bytes();
            frame[..2].copy_from_slice(&bytes);
            frame[2..].copy_from_slice(&bytes);
            self.phase = self.phase.wrapping_add(self.step);
        }
    }
}

/// High-pass coefficient for a ~100 Hz corner at 16 kHz, in Q15.
const HP_ALPHA_Q15: i64 = 31_507;
/// Low-pass coefficient for a ~6.3 kHz corner at 16 kHz, in Q15.
const LP_ALPHA_Q15: i64 = 30_000;

/// A stateful voice-band filter for clip playback.
///
/// Removes DC/rumble and high-frequency hiss without touching the generated
/// tone path. The output is duplicated to both slots, matching [`fill_playback`].
#[derive(Default)]
pub struct PlaybackFilter {
    hp_prev_in: i32,
    hp_prev_out: i32,
    lp_prev: i32,
}

impl PlaybackFilter {
    /// A filter with zeroed history.
    pub const fn new() -> Self {
        Self {
            hp_prev_in: 0,
            hp_prev_out: 0,
            lp_prev: 0,
        }
    }

    /// Filters stereo frames in place, duplicating the left sample to the right.
    pub fn apply(&mut self, out: &mut [u8]) {
        for frame in out.chunks_exact_mut(4) {
            let sample = i16::from_le_bytes([frame[0], frame[1]]);
            let filtered = self.filter(sample).to_le_bytes();
            frame[..2].copy_from_slice(&filtered);
            frame[2..].copy_from_slice(&filtered);
        }
    }

    fn filter(&mut self, sample: i16) -> i16 {
        let input = i32::from(sample);
        let high_passed =
            ((HP_ALPHA_Q15 * i64::from(self.hp_prev_out + input - self.hp_prev_in)) >> 15) as i32;
        self.hp_prev_in = input;
        self.hp_prev_out = high_passed;

        let low_passed =
            self.lp_prev + ((LP_ALPHA_Q15 * i64::from(high_passed - self.lp_prev)) >> 15) as i32;
        self.lp_prev = low_passed;
        low_passed.clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16
    }
}

/// Copies one channel from a stereo clip into stereo output, duplicating it.
///
/// Returns the number of frames written.
pub fn fill_playback(
    clip: &[u8],
    start_frame: usize,
    channel_offset: usize,
    out: &mut [u8],
) -> usize {
    let frames = out.len() / 4;
    let mut written = 0;
    for frame in out.chunks_exact_mut(4) {
        let source = (start_frame + written) * 4 + channel_offset;
        if source + 1 >= clip.len() {
            break;
        }
        let sample = &clip[source..source + 2];
        frame[..2].copy_from_slice(sample);
        frame[2..].copy_from_slice(sample);
        written += 1;
    }
    frames.min(written)
}

fn integer_sqrt(value: u64) -> u16 {
    if value == 0 {
        return 0;
    }
    let mut root = value;
    let mut next = root.div_ceil(2);
    while next < root {
        root = next;
        next = (root + value / root) / 2;
    }
    root.min(u64::from(u16::MAX)) as u16
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::vec::Vec;

    fn tone_clip(frequency_hz: u16, frames: usize) -> Vec<u8> {
        let mut tone = Tone::new(frequency_hz);
        let mut clip = vec![0u8; frames * 4];
        tone.fill(&mut clip);
        clip
    }

    #[test]
    fn frequency_estimation_matches_the_generator() {
        let clip = tone_clip(1_000, 16_000);
        assert!((estimate_hz(&clip, 0) as i32 - 1_000).abs() <= 1);
    }

    #[test]
    fn levels_find_peak_and_clipping() {
        let mut levels = Levels::new();
        levels.push(32_767, -16_000);
        levels.push(0, 0);
        let (left, right) = levels.snapshot();
        assert!(left.clipped);
        assert_eq!(left.peak, 32_767);
        assert_eq!(right.peak, 16_000);
    }

    #[test]
    fn playback_filter_removes_dc_and_duplicates() {
        let mut clip = vec![0u8; 4 * 400];
        for frame in clip.chunks_exact_mut(4) {
            frame[..2].copy_from_slice(&10_000i16.to_le_bytes());
            frame[2..].copy_from_slice(&10_000i16.to_le_bytes());
        }
        let mut filter = PlaybackFilter::new();
        filter.apply(&mut clip);
        let last = &clip[clip.len() - 4..];
        assert!(i16::from_le_bytes([last[0], last[1]]).abs() < 100);
        assert_eq!(last[..2], last[2..]);
    }

    #[test]
    fn playback_filter_passes_voice_band_energy() {
        let mut tone = Tone::new(1_000);
        let mut original = vec![0u8; 4 * 1_600];
        tone.fill(&mut original);
        let mut filtered = original.clone();
        let mut filter = PlaybackFilter::new();
        filter.apply(&mut filtered);

        let rms = |clip: &[u8]| -> u64 {
            clip.chunks_exact(4)
                .map(|frame| {
                    let sample = i64::from(i16::from_le_bytes([frame[0], frame[1]]));
                    (sample * sample) as u64
                })
                .sum::<u64>()
                / (clip.len() / 4) as u64
        };
        let before = rms(&original);
        let after = rms(&filtered);
        assert!(after * 100 / before > 80);
    }

    #[test]
    fn playback_duplicates_the_selected_channel() {
        let mut clip = vec![0u8; 4 * 2];
        clip[..2].copy_from_slice(&100i16.to_le_bytes());
        clip[2..4].copy_from_slice(&200i16.to_le_bytes());
        clip[4..6].copy_from_slice(&300i16.to_le_bytes());
        clip[6..8].copy_from_slice(&400i16.to_le_bytes());
        let mut out = [0u8; 8];
        assert_eq!(fill_playback(&clip, 0, 2, &mut out), 2);
        assert_eq!(i16::from_le_bytes([out[0], out[1]]), 200);
        assert_eq!(i16::from_le_bytes([out[4], out[5]]), 400);
    }
}
