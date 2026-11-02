//! Command/event bridge and task for the Audio Lab.

use core::future::Future;
use core::sync::atomic::{AtomicBool, Ordering};

use common::Cpu1State;
use embassy_executor::task;
use embassy_sync08::channel::Channel;
use embassy_time::{Duration, Instant, Timer};
use esp_sync::RawMutex;
use voice::lab::LabCommand;

pub use voice::lab::{LabError, LabEvent, LabIntent};

use super::audio::{Clip, Transport, TransportError};

/// Event queue depth.
pub const EVENT_CAPACITY: usize = 16;
/// Command queue depth.
const COMMAND_CAPACITY: usize = 8;

static COMMANDS: Channel<RawMutex, LabCommand, COMMAND_CAPACITY> = Channel::new();
static EVENTS: Channel<RawMutex, LabEvent, EVENT_CAPACITY> = Channel::new();
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

/// Stops the worker and waits until it has drained the transport.
pub async fn stop_and_wait() {
    stop();
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        while let Ok(event) = EVENTS.try_receive() {
            if matches!(event, LabEvent::Stopped) {
                Timer::after(Duration::from_millis(2)).await;
                return;
            }
        }
        if Instant::now() >= deadline {
            return;
        }
        Timer::after(Duration::from_millis(2)).await;
    }
}

/// Removes the next event for the app.
pub fn try_event() -> Option<LabEvent> {
    EVENTS.try_receive().ok()
}

/// The audio worker. Owns the I2S transport and the preallocated clip.
#[task]
pub async fn task(transport: Transport, clip: Clip) {
    crate::metrics::set_cpu1_state(Cpu1State::Idle);
    metered(worker(transport, clip)).await;
}

async fn worker(mut transport: Transport, mut clip: Clip) {
    let receiver = COMMANDS.receiver();
    let sender = EVENTS.sender();
    loop {
        let command = receiver.receive().await;
        crate::metrics::set_cpu1_state(Cpu1State::Audio);
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
                } else {
                    let mut emit = |event| {
                        let _ = sender.try_send(event);
                    };
                    let result = transport
                        .play_clip(&clip, channel, &CANCEL, &mut emit)
                        .await;
                    send_finished(&sender, result);
                }
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
        crate::metrics::set_cpu1_state(Cpu1State::Idle);
    }
}

async fn metered<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    core::future::poll_fn(|cx| {
        let start = Instant::now();
        let poll = future.as_mut().poll(cx);
        let micros = (Instant::now() - start).as_micros();
        crate::metrics::add_cpu1_busy_us(micros.min(u64::from(u32::MAX)) as u32);
        poll
    })
    .await
}

fn send_finished(
    sender: &embassy_sync08::channel::Sender<'_, RawMutex, LabEvent, EVENT_CAPACITY>,
    result: Result<(), TransportError>,
) {
    if CANCEL.load(Ordering::Acquire) {
        let _ = sender.try_send(LabEvent::Stopped);
    } else {
        send_result(sender, result.map(|_| LabEvent::Finished));
    }
}

fn send_result(
    sender: &embassy_sync08::channel::Sender<'_, RawMutex, LabEvent, EVENT_CAPACITY>,
    result: Result<LabEvent, TransportError>,
) {
    match result {
        Ok(event) => {
            let _ = sender.try_send(event);
        }
        Err(error) => {
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
