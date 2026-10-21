//! Command/event bridge and task for the Audio Lab.

use core::sync::atomic::{AtomicBool, Ordering};

use embassy_executor::task;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use esp_alloc::ExternalMemory;
use esp_println::{print, println};
use voice::lab::LabCommand;

pub use voice::lab::{LabError, LabEvent, LabIntent};

use super::audio::{Clip, Transport, TransportError};

/// Event queue depth.
pub const EVENT_CAPACITY: usize = 16;
/// Command queue depth.
const COMMAND_CAPACITY: usize = 8;
/// Temporary capture debugging: dump the start of the last clip over serial.
const DUMP_CAPTURE: bool = true;
/// Frames dumped by [`dump_clip`].
const DUMP_FRAMES: usize = 8_000;
/// Frames per hex line in the dump.
const DUMP_FRAMES_PER_LINE: usize = 8;

static COMMANDS: Channel<CriticalSectionRawMutex, LabCommand, COMMAND_CAPACITY> = Channel::new();
static EVENTS: Channel<CriticalSectionRawMutex, LabEvent, EVENT_CAPACITY> = Channel::new();
static CANCEL: AtomicBool = AtomicBool::new(false);

/// A rejected control write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlError {
    /// The codec rejected the value.
    Codec,
    /// The gain is not a valid 3 dB step.
    Gain,
}

/// Sends one command to the audio task.
pub fn command(command: LabCommand) -> bool {
    CANCEL.store(false, Ordering::Release);
    while COMMANDS.try_receive().is_ok() {}
    COMMANDS.try_send(command).is_ok()
}

/// Cancels the running operation and parks the task.
pub fn stop() {
    CANCEL.store(true, Ordering::Release);
    while COMMANDS.try_receive().is_ok() {}
    let _ = COMMANDS.try_send(LabCommand::Stop);
}

/// Removes the next event for the app.
pub fn try_event() -> Option<LabEvent> {
    EVENTS.try_receive().ok()
}

/// The audio worker. Owns the I2S transport and the last clip.
#[task]
pub async fn task(mut transport: Transport) {
    let receiver = COMMANDS.receiver();
    let sender = EVENTS.sender();
    let mut clip = Clip::new_in(ExternalMemory);
    loop {
        let command = receiver.receive().await;
        println!("[audio] {:?}", command);
        CANCEL.store(false, Ordering::Release);
        match command {
            LabCommand::Stop => {
                let _ = sender.try_send(LabEvent::Stopped);
            }
            LabCommand::Record {
                duration_ms,
                tone_hz,
            } => {
                let mut emit = |event| {
                    let _ = sender.try_send(event);
                };
                let result = transport
                    .record(duration_ms, tone_hz, &mut clip, &CANCEL, &mut emit)
                    .await;
                if CANCEL.load(Ordering::Acquire) {
                    let _ = sender.try_send(LabEvent::Stopped);
                } else {
                    send_result(&sender, result.map(LabEvent::Recorded));
                }
            }
            LabCommand::PlayClip { channel } => {
                if clip.is_empty() {
                    let _ = sender.try_send(LabEvent::Error(LabError::NoClip));
                    continue;
                }
                let mut emit = |event| {
                    let _ = sender.try_send(event);
                };
                let result = transport
                    .play_clip(&clip, channel, &CANCEL, &mut emit)
                    .await;
                send_finished(&sender, result);
            }
            LabCommand::PlayTone {
                frequency_hz,
                duration_ms,
            } => {
                let mut emit = |event| {
                    let _ = sender.try_send(event);
                };
                let result = transport
                    .play_tone(frequency_hz, duration_ms, &CANCEL, &mut emit)
                    .await;
                send_finished(&sender, result);
            }
        }
    }
}

/// Prints the first frames of a clip as `LRLR...` hex, eight frames per line.
#[allow(dead_code)]
fn dump_clip(clip: &[u8]) {
    if !DUMP_CAPTURE {
        return;
    }
    let frames = (clip.len() / 4).min(DUMP_FRAMES);
    println!(
        "[pcm] begin frames={} channels=2 rate={}",
        frames,
        voice::lab::SAMPLE_RATE_HZ
    );
    for (index, frame) in clip.chunks_exact(4).take(frames).enumerate() {
        let left = u16::from_le_bytes([frame[0], frame[1]]);
        let right = u16::from_le_bytes([frame[2], frame[3]]);
        print!("{:04X}{:04X}", left, right);
        if (index + 1) % DUMP_FRAMES_PER_LINE == 0 {
            println!();
        }
    }
    if !frames.is_multiple_of(DUMP_FRAMES_PER_LINE) {
        println!();
    }
    println!("[pcm] end");
}

fn send_finished(
    sender: &embassy_sync::channel::Sender<'_, CriticalSectionRawMutex, LabEvent, EVENT_CAPACITY>,
    result: Result<(), TransportError>,
) {
    if CANCEL.load(Ordering::Acquire) {
        let _ = sender.try_send(LabEvent::Stopped);
    } else {
        send_result(sender, result.map(|_| LabEvent::Finished));
    }
}

fn send_result(
    sender: &embassy_sync::channel::Sender<'_, CriticalSectionRawMutex, LabEvent, EVENT_CAPACITY>,
    result: Result<LabEvent, TransportError>,
) {
    match result {
        Ok(event) => {
            let _ = sender.try_send(event);
        }
        Err(error) => {
            println!("[audio] transport error: {:?}", error);
            let _ = sender.try_send(LabEvent::Error(map_error(error)));
        }
    }
}

fn map_error(error: TransportError) -> LabError {
    match error {
        TransportError::Alloc => LabError::Alloc,
        TransportError::Empty => LabError::Empty,
        TransportError::Config(_) | TransportError::I2s(_) => LabError::I2s,
        TransportError::Dma(error) => match error {
            esp_hal::dma::DmaError::Late => LabError::DmaLate,
            esp_hal::dma::DmaError::InvalidAlignment(_) => LabError::DmaAlign,
            esp_hal::dma::DmaError::OutOfDescriptors => LabError::DmaDescriptors,
            esp_hal::dma::DmaError::DescriptorError => LabError::DmaDescriptor,
            esp_hal::dma::DmaError::Overflow => LabError::DmaOverflow,
            esp_hal::dma::DmaError::BufferTooSmall => LabError::DmaTooSmall,
            esp_hal::dma::DmaError::UnsupportedMemoryRegion => LabError::DmaMemory,
            esp_hal::dma::DmaError::InvalidChunkSize => LabError::DmaChunk,
        },
    }
}
