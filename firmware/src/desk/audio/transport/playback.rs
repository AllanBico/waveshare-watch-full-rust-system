//! Clip and tone playback.

use core::sync::atomic::Ordering;

use embassy_time::{Duration, Instant, Timer};
use esp_println::println;
use voice::lab::{LabEvent, MicChannel};

use super::super::analysis::{self, PlaybackFilter, Tone};
use super::{frames_for, PaGuard, Transport, TransportError, METER_MS, PA_SETTLE_MS, POLL_MS};

impl Transport {
    /// Plays a recorded clip, duplicating one channel to both slots.
    pub async fn play_clip<F>(
        &mut self,
        clip: &[u8],
        channel: MicChannel,
        cancel: &core::sync::atomic::AtomicBool,
        emit: &mut F,
    ) -> Result<(), TransportError>
    where
        F: FnMut(LabEvent),
    {
        let frames = clip.len() / 4;
        if frames == 0 {
            return Err(TransportError::Empty);
        }
        let offset = match channel {
            MicChannel::One => 0,
            MicChannel::Two => 2,
        };
        let duration_ms = (frames as u64 * 1_000 / u64::from(voice::lab::SAMPLE_RATE_HZ)) as u16;
        let Self {
            tx,
            tx_buffer,
            scratch,
            pa,
            ..
        } = self;
        let ring_frames = tx_buffer.len() / 4;
        let first = frames.min(ring_frames);
        let mut filter = PlaybackFilter::new();
        analysis::fill_playback(clip, 0, offset, &mut tx_buffer[..first * 4]);
        filter.apply(&mut tx_buffer[..first * 4]);
        tx_buffer[first * 4..].fill(0);
        let stage = &mut scratch[..];

        let pa = PaGuard(pa);
        pa.0.set_high();
        Timer::after(Duration::from_millis(PA_SETTLE_MS)).await;
        let buffer: &[u8] = &tx_buffer[..];
        let mut transfer = tx.write_dma_circular(&buffer)?;
        emit(LabEvent::Playback {
            elapsed_ms: 0,
            duration_ms,
        });
        let started = Instant::now();
        let deadline = started + Duration::from_millis(u64::from(duration_ms));
        let mut next = first;
        let mut last_emit = started;

        while next < frames && !cancel.load(Ordering::Acquire) {
            let available = transfer.available()?;
            if available == 0 {
                Timer::after(Duration::from_millis(POLL_MS)).await;
                continue;
            }
            let take_frames = (available.min(stage.len())) / 4;
            let written =
                analysis::fill_playback(clip, next, offset, &mut stage[..take_frames * 4]);
            if written == 0 {
                break;
            }
            filter.apply(&mut stage[..written * 4]);
            transfer.push(&stage[..written * 4])?;
            next += written;
            Timer::after(Duration::from_millis(1)).await;
            if (Instant::now() - last_emit).as_millis() >= METER_MS {
                last_emit = Instant::now();
                emit(LabEvent::Playback {
                    elapsed_ms: (Instant::now() - started)
                        .as_millis()
                        .min(u64::from(u16::MAX)) as u16,
                    duration_ms,
                });
            }
        }
        while Instant::now() < deadline && !cancel.load(Ordering::Acquire) {
            Timer::after(Duration::from_millis(POLL_MS)).await;
        }
        transfer.stop()?;
        println!("[audio] clip done");
        Ok(())
    }

    /// Plays a generated stereo tone.
    pub async fn play_tone<F>(
        &mut self,
        frequency_hz: u16,
        duration_ms: u16,
        cancel: &core::sync::atomic::AtomicBool,
        emit: &mut F,
    ) -> Result<(), TransportError>
    where
        F: FnMut(LabEvent),
    {
        let Self {
            tx,
            tx_buffer,
            scratch,
            pa,
            ..
        } = self;
        let mut tone = Tone::new(frequency_hz);
        tone.fill(&mut tx_buffer[..]);
        let total_frames = frames_for(duration_ms);
        let ring_frames = tx_buffer.len() / 4;
        let stage = &mut scratch[..];

        let pa = PaGuard(pa);
        pa.0.set_high();
        Timer::after(Duration::from_millis(PA_SETTLE_MS)).await;
        let buffer: &[u8] = &tx_buffer[..];
        let mut transfer = tx.write_dma_circular(&buffer)?;
        emit(LabEvent::Playback {
            elapsed_ms: 0,
            duration_ms,
        });
        let started = Instant::now();
        let deadline = started + Duration::from_millis(u64::from(duration_ms));
        let mut generated = ring_frames.min(total_frames);

        while Instant::now() < deadline && !cancel.load(Ordering::Acquire) {
            if generated < total_frames {
                let available = transfer.available()?;
                if available == 0 {
                    Timer::after(Duration::from_millis(POLL_MS)).await;
                    continue;
                }
                let take_frames = (available.min(stage.len())) / 4;
                let frames = take_frames.min(total_frames - generated);
                if frames == 0 {
                    Timer::after(Duration::from_millis(POLL_MS)).await;
                    continue;
                }
                tone.fill(&mut stage[..frames * 4]);
                transfer.push(&stage[..frames * 4])?;
                generated += frames;
            }
            Timer::after(Duration::from_millis(1)).await;
        }
        transfer.stop()?;
        println!("[audio] tone done");
        Ok(())
    }
}
