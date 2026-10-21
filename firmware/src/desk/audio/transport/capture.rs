//! Stereo capture with optional test tone playback.
//!
//! The RX ring is deliberately large: the desk UI can hold the executor for a
//! few hundred milliseconds while it flushes a frame. A smaller ring overran
//! and returned `DmaError::Late`; this one covers the measured worst frame.

use core::sync::atomic::Ordering;

use embassy_time::{Duration, Instant, Timer};
use voice::lab::{ClipReport, LabEvent};

use super::super::analysis::{self, Levels, Tone};
use super::{
    frames_for, Clip, PaGuard, Transport, TransportError, METER_MS, PA_SETTLE_MS, POLL_MS,
    PRIME_FRAMES, PRIME_TIMEOUT_MS,
};

impl Transport {
    /// Captures stereo audio, optionally playing a test tone.
    pub async fn record<F>(
        &mut self,
        duration_ms: u16,
        tone_hz: u16,
        clip: &mut Clip,
        cancel: &core::sync::atomic::AtomicBool,
        emit: &mut F,
    ) -> Result<ClipReport, TransportError>
    where
        F: FnMut(LabEvent),
    {
        let total = frames_for(duration_ms) * 4;
        clip.clear();
        clip.try_reserve_exact(total)
            .map_err(|_| TransportError::Alloc)?;
        clip.resize(total, 0);
        let Self {
            tx,
            rx,
            tx_buffer,
            rx_buffer,
            scratch,
            pa,
            ..
        } = self;
        let pa = PaGuard(pa);
        if tone_hz != 0 {
            pa.0.set_high();
            Timer::after(Duration::from_millis(PA_SETTLE_MS)).await;
        }
        tx_buffer.fill(0);
        if tone_hz != 0 {
            Tone::new(tone_hz).fill(&mut tx_buffer[..]);
        }
        let buffer: &[u8] = &tx_buffer[..];
        let tx_hold = tx.write_dma_circular(&buffer)?;
        let mut rx_buffer_view: &mut [u8] = &mut rx_buffer[..];
        let mut rx_transfer = rx.read_dma_circular(&mut rx_buffer_view)?;

        // Drop the first frames so the capture starts on a settled, whole
        // stereo frame rather than whatever the RX was doing at enable time.
        let mut primed = 0usize;
        let prime_deadline = Instant::now() + Duration::from_millis(PRIME_TIMEOUT_MS);
        while primed < PRIME_FRAMES && Instant::now() < prime_deadline {
            let available = match rx_transfer.available() {
                Ok(value) => value,
                Err(error) => {
                    return Err(error.into());
                }
            };
            if available == 0 {
                Timer::after(Duration::from_millis(POLL_MS)).await;
                continue;
            }
            let take = available.min(scratch.len());
            let popped = match rx_transfer.pop(&mut scratch[..take]) {
                Ok(value) => value,
                Err(error) => {
                    return Err(error.into());
                }
            };
            primed += popped / 4;
        }
        if primed == 0 {
            return Err(TransportError::Empty);
        }

        emit(LabEvent::Started {
            duration_ms,
            tone_hz,
        });
        let started = Instant::now();
        let deadline = started + Duration::from_millis(u64::from(duration_ms));
        let mut last_meter = started;
        let mut levels = Levels::new();
        let mut written = 0usize;
        let mut mic1_hz = 0u16;
        let mut mic2_hz = 0u16;

        while Instant::now() < deadline && !cancel.load(Ordering::Acquire) {
            let available = match rx_transfer.available() {
                Ok(value) => value,
                Err(error) => {
                    return Err(error.into());
                }
            };
            if available == 0 {
                Timer::after(Duration::from_millis(POLL_MS)).await;
                continue;
            }
            let take = available.min(scratch.len());
            let popped = match rx_transfer.pop(&mut scratch[..take]) {
                Ok(value) => value,
                Err(error) => {
                    return Err(error.into());
                }
            };
            let copy = popped.min(clip.len().saturating_sub(written));
            clip[written..written + copy].copy_from_slice(&scratch[..copy]);
            written += copy;
            for frame in scratch[..popped].chunks_exact(4) {
                levels.push(
                    i16::from_le_bytes([frame[0], frame[1]]),
                    i16::from_le_bytes([frame[2], frame[3]]),
                );
            }
            let chunk1_hz = analysis::estimate_hz(&scratch[..popped], 0);
            let chunk2_hz = analysis::estimate_hz(&scratch[..popped], 2);
            if chunk1_hz != 0 {
                mic1_hz = chunk1_hz;
            }
            if chunk2_hz != 0 {
                mic2_hz = chunk2_hz;
            }
            if (Instant::now() - last_meter).as_millis() >= METER_MS {
                last_meter = Instant::now();
                let (mic1, mic2) = levels.snapshot();
                emit(LabEvent::Recording {
                    elapsed_ms: (Instant::now() - started)
                        .as_millis()
                        .min(u64::from(u16::MAX)) as u16,
                    mic1,
                    mic2,
                });
            }
        }

        drop(rx_transfer);
        tx_hold.stop()?;
        drop(pa);
        clip.truncate(written);
        let elapsed_ms = (Instant::now() - started)
            .as_millis()
            .min(u64::from(u32::MAX)) as u32;
        let mic1 = analysis::meter(clip, 0);
        let mic2 = analysis::meter(clip, 2);
        Ok(ClipReport {
            frames: (written / 4) as u32,
            elapsed_ms,
            mic1,
            mic2,
            mic1_hz,
            mic2_hz,
            mic1_dc: analysis::dc_offset(clip, 0),
            mic2_dc: analysis::dc_offset(clip, 2),
            tone_hz,
        })
    }
}
