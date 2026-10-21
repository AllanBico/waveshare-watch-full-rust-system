//! Fixed-buffer text for the Audio Lab screens.

use voice::lab::{ClipReport, Meter};

/// Formats a peak meter as `-12 dB`.
pub(super) fn peak_text(meter: Meter, out: &mut [u8; 12]) -> &str {
    let db = meter.peak_dbfs();
    let mut index = 0;
    if db < 0 {
        out[index] = b'-';
        index += 1;
    }
    index = put_u32(db.unsigned_abs() as u32, out, index);
    out[index..index + 3].copy_from_slice(b" dB");
    text(out, index + 3)
}

/// Formats a prefixed microphone meter as `MIC1 -12 dB`.
pub(super) fn mic_text<'a>(prefix: &'a str, meter: Meter, out: &'a mut [u8; 20]) -> &'a str {
    let prefix = prefix.as_bytes();
    let len = prefix.len().min(out.len());
    out[..len].copy_from_slice(&prefix[..len]);
    let mut tail = [0u8; 12];
    let peak = peak_text(meter, &mut tail);
    let peak = peak.as_bytes();
    let tail_len = peak.len().min(out.len().saturating_sub(len));
    out[len..len + tail_len].copy_from_slice(&peak[..tail_len]);
    text(out, len + tail_len)
}

/// Formats a percentage as `80%`.
pub(super) fn percent_text(value: u8, out: &mut [u8; 6]) -> &str {
    let index = put_u32(u32::from(value), out, 0);
    out[index] = b'%';
    text(out, index + 1)
}

/// Formats a gain as `18 dB`.
pub(super) fn gain_text(db: u8, out: &mut [u8; 8]) -> &str {
    let index = put_u32(u32::from(db), out, 0);
    out[index..index + 3].copy_from_slice(b" dB");
    text(out, index + 3)
}

/// Formats a duration as `5000 ms`.
pub(super) fn duration_text(ms: u16, out: &mut [u8; 10]) -> &str {
    let index = put_u32(u32::from(ms), out, 0);
    out[index..index + 3].copy_from_slice(b" ms");
    text(out, index + 3)
}

/// Formats a frequency, using `--` for an unknown tone.
pub(super) fn hz_text(hz: u16, out: &mut [u8; 10]) -> &str {
    if hz == 0 {
        out[..2].copy_from_slice(b"--");
        return text(out, 2);
    }
    let index = put_u32(u32::from(hz), out, 0);
    out[index..index + 3].copy_from_slice(b" Hz");
    text(out, index + 3)
}

/// Formats a plain count.
pub(super) fn count_text(value: u32, out: &mut [u8; 11]) -> &str {
    let index = put_u32(value, out, 0);
    text(out, index)
}

/// Formats a rate percentage.
pub(super) fn rate_text(report: ClipReport, out: &mut [u8; 6]) -> &str {
    percent_text(report.rate_percent().min(255) as u8, out)
}

/// Formats a signed DC offset.
pub(super) fn dc_text(value: i16, out: &mut [u8; 8]) -> &str {
    let mut index = 0;
    if value < 0 {
        out[index] = b'-';
        index += 1;
    }
    index = put_u32(value.unsigned_abs() as u32, out, index);
    text(out, index)
}

/// Builds the one-line detail caption for a page.
pub(super) fn detail_text<'a>(
    page: u8,
    rate: &'a str,
    hz1: &'a str,
    hz2: &'a str,
    dc1: &'a str,
    dc2: &'a str,
    out: &'a mut [u8; 64],
) -> &'a str {
    let mut index = 0;
    if page == 0 {
        index = push(out, index, "Rate ");
        index = push(out, index, rate);
        index = push(out, index, "  M1 ");
        index = push(out, index, hz1);
        index = push(out, index, "  M2 ");
        index = push(out, index, hz2);
    } else if page == 1 {
        index = push(out, index, "M1 ");
        index = push(out, index, hz1);
        index = push(out, index, "  M2 ");
        index = push(out, index, hz2);
        index = push(out, index, "  DC ");
        index = push(out, index, dc1);
        index = push(out, index, "/");
        index = push(out, index, dc2);
    } else {
        index = push(out, index, "Tone 440 / 1k");
    }
    text(out, index)
}

fn push(out: &mut [u8], index: usize, value: &str) -> usize {
    let bytes = value.as_bytes();
    let len = bytes.len().min(out.len().saturating_sub(index));
    out[index..index + len].copy_from_slice(&bytes[..len]);
    index + len
}

/// Formats the operation status line.
pub(super) fn status_text(
    is_recording: bool,
    is_playing: bool,
    elapsed_ms: u16,
    out: &mut [u8; 20],
) -> &str {
    let label: &[u8] = if is_recording {
        b"Recording "
    } else if is_playing {
        b"Playing "
    } else {
        out[..4].copy_from_slice(b"Idle");
        return text(out, 4);
    };
    out[..label.len()].copy_from_slice(label);
    let index = put_u32(u32::from(elapsed_ms / 1_000), out, label.len());
    out[index] = b'.';
    out[index + 1] = b'0' + (elapsed_ms / 100 % 10) as u8;
    out[index + 2] = b's';
    text(out, index + 3)
}

fn put_u32(value: u32, out: &mut [u8], mut index: usize) -> usize {
    let mut digits = [0u8; 10];
    let mut count = 0;
    let mut value = value;
    loop {
        digits[count] = b'0' + (value % 10) as u8;
        count += 1;
        value /= 10;
        if value == 0 || count == digits.len() {
            break;
        }
    }
    while count > 0 && index < out.len() {
        count -= 1;
        out[index] = digits[count];
        index += 1;
    }
    index
}

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peak_is_signed() {
        let mut buffer = [0; 12];
        assert_eq!(
            peak_text(
                Meter {
                    peak: 16_384,
                    ..Meter::SILENT
                },
                &mut buffer
            ),
            "-6 dB"
        );
        assert_eq!(
            peak_text(
                Meter {
                    peak: 32_768,
                    ..Meter::SILENT
                },
                &mut buffer
            ),
            "0 dB"
        );
    }

    #[test]
    fn frequency_uses_placeholder() {
        let mut buffer = [0; 10];
        assert_eq!(hz_text(0, &mut buffer), "--");
        assert_eq!(hz_text(1_000, &mut buffer), "1000 Hz");
    }

    #[test]
    fn status_shows_tenths() {
        let mut buffer = [0; 20];
        assert_eq!(
            status_text(true, false, 1_250, &mut buffer),
            "Recording 1.2s"
        );
        assert_eq!(status_text(false, true, 350, &mut buffer), "Playing 0.3s");
        assert_eq!(status_text(false, false, 0, &mut buffer), "Idle");
    }
}

#[cfg(test)]
mod detail_tests {
    use super::*;

    #[test]
    fn loop_detail_leads_with_rate() {
        let mut buffer = [0; 64];
        assert_eq!(
            detail_text(0, "100%", "1000 Hz", "950 Hz", "0", "0", &mut buffer),
            "Rate 100%  M1 1000 Hz  M2 950 Hz"
        );
    }
}
