//! Audio Lab state, intents, event ingest, and frame orchestration.

use common::{Mode, Region};
use shell::{App, AppContext, AppError};
use ui::{
    paint_damage_shaped, paint_widgets, Damage, HitRegistry, RedrawReason, Scene, SceneArena,
    SceneError, SceneNode, Surface, TextCache, Theme, WidgetArena,
};
use voice::lab::{
    ClipReport, LabCommand, LabError, LabEvent, LabIntent, Meter, MicChannel, DEFAULT_RECORD_MS,
    GAIN_STEP_DB, MAX_GAIN_DB,
};

use super::format;
use super::view;

const RENDER_FAILED_CODE: u16 = 1;
const INTENT_CAPACITY: usize = 8;
const DURATIONS_MS: [u16; 3] = [3_000, 5_000, 10_000];

/// Which page the app shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Page {
    Loopback,
    Mic,
    Speaker,
}

impl Page {
    pub(super) const fn index(self) -> u8 {
        match self {
            Self::Loopback => 0,
            Self::Mic => 1,
            Self::Speaker => 2,
        }
    }
}

/// A frame failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// A screen could not be built or laid out.
    Build,
    /// The scene arena overflowed.
    Scene(SceneError),
}

/// The desk-only audio tuning app.
pub struct AudioLab {
    mode: Mode,
    pub(super) page: Page,
    pub(super) record_ms: u16,
    pub(super) gain_db: u8,
    pub(super) volume: u8,
    pub(super) muted: bool,
    pub(super) channel: MicChannel,
    pub(super) is_recording: bool,
    pub(super) is_playing: bool,
    pub(super) elapsed_ms: u16,
    pub(super) duration_ms: u16,
    pub(super) mic1: Meter,
    pub(super) mic2: Meter,
    pub(super) report: Option<ClipReport>,
    pub(super) error: Option<LabError>,
    intents: [Option<LabIntent>; INTENT_CAPACITY],
    intent_len: usize,
    pub(super) hits: HitRegistry,
    cache: Option<TextCache>,
    scene: Scene,
    pub(super) status_text: [u8; 20],
    pub(super) status_len: usize,
    pub(super) mic1_text: [u8; 20],
    pub(super) mic1_len: usize,
    pub(super) mic2_text: [u8; 20],
    pub(super) mic2_len: usize,
    pub(super) gain_text: [u8; 8],
    pub(super) gain_len: usize,
    pub(super) volume_text: [u8; 6],
    pub(super) volume_len: usize,
    pub(super) duration_text: [u8; 10],
    pub(super) duration_len: usize,
    pub(super) rate_text: [u8; 6],
    pub(super) rate_len: usize,
    pub(super) hz1_text: [u8; 10],
    pub(super) hz1_len: usize,
    pub(super) hz2_text: [u8; 10],
    pub(super) hz2_len: usize,
    pub(super) frames_text: [u8; 11],
    pub(super) frames_len: usize,
    pub(super) dc1_text: [u8; 8],
    pub(super) dc1_len: usize,
    pub(super) dc2_text: [u8; 8],
    pub(super) dc2_len: usize,
    pub(super) error_text: [u8; 20],
    pub(super) error_len: usize,
    pub(super) detail_text: [u8; 64],
    pub(super) detail_len: usize,
}

impl AudioLab {
    /// Creates the app in its idle state.
    pub fn new(mode: Mode) -> Self {
        let mut app = Self {
            mode,
            page: Page::Loopback,
            record_ms: DEFAULT_RECORD_MS,
            gain_db: 24,
            volume: 80,
            muted: false,
            channel: MicChannel::One,
            is_recording: false,
            is_playing: false,
            elapsed_ms: 0,
            duration_ms: 0,
            mic1: Meter::SILENT,
            mic2: Meter::SILENT,
            report: None,
            error: None,
            intents: [None; INTENT_CAPACITY],
            intent_len: 0,
            hits: HitRegistry::new(),
            cache: Some(TextCache::new()),
            scene: Scene::new(),
            status_text: [0; 20],
            status_len: 0,
            mic1_text: [0; 20],
            mic1_len: 0,
            mic2_text: [0; 20],
            mic2_len: 0,
            gain_text: [0; 8],
            gain_len: 0,
            volume_text: [0; 6],
            volume_len: 0,
            duration_text: [0; 10],
            duration_len: 0,
            rate_text: [0; 6],
            rate_len: 0,
            hz1_text: [0; 10],
            hz1_len: 0,
            hz2_text: [0; 10],
            hz2_len: 0,
            frames_text: [0; 11],
            frames_len: 0,
            dc1_text: [0; 8],
            dc1_len: 0,
            dc2_text: [0; 8],
            dc2_len: 0,
            error_text: [0; 20],
            error_len: 0,
            detail_text: [0; 64],
            detail_len: 0,
        };
        app.refresh_text();
        app
    }

    /// Whether a capture or playback is in flight.
    pub const fn is_active(&self) -> bool {
        self.is_recording || self.is_playing
    }

    /// Removes the oldest intent queued for the firmware.
    pub fn take_intent(&mut self) -> Option<LabIntent> {
        if self.intent_len == 0 {
            return None;
        }
        let item = self.intents[0].take();
        for index in 1..self.intent_len {
            self.intents[index - 1] = self.intents[index].take();
        }
        self.intent_len -= 1;
        item
    }

    /// Folds one firmware report into the app.
    pub fn ingest(&mut self, event: LabEvent) {
        match event {
            LabEvent::Started {
                duration_ms,
                tone_hz,
            } => {
                self.error = None;
                self.elapsed_ms = 0;
                self.duration_ms = duration_ms;
                self.is_recording = true;
                self.is_playing = tone_hz != 0;
            }
            LabEvent::Recording {
                elapsed_ms,
                mic1,
                mic2,
            } => {
                self.is_recording = true;
                self.elapsed_ms = elapsed_ms;
                self.mic1 = mic1;
                self.mic2 = mic2;
            }
            LabEvent::Recorded(report) => {
                self.is_recording = false;
                self.is_playing = false;
                self.report = Some(report);
                self.elapsed_ms = report.elapsed_ms.min(u32::from(u16::MAX)) as u16;
                self.mic1 = report.mic1;
                self.mic2 = report.mic2;
            }
            LabEvent::Playback {
                elapsed_ms,
                duration_ms,
            } => {
                self.error = None;
                self.is_playing = true;
                self.elapsed_ms = elapsed_ms;
                self.duration_ms = duration_ms;
            }
            LabEvent::Finished => {
                self.is_recording = false;
                self.is_playing = false;
            }
            LabEvent::Stopped => {
                self.is_recording = false;
                self.is_playing = false;
            }
            LabEvent::Error(error) => {
                self.is_recording = false;
                self.is_playing = false;
                self.error = Some(error);
            }
        }
        self.refresh_text();
    }

    /// Builds, diffs, and paints the current page.
    pub fn render_frame(
        &mut self,
        surface: &mut dyn Surface,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, RenderError> {
        self.refresh_text();
        let mut cache = self.cache.take().unwrap_or_default();
        let mut hits = core::mem::replace(&mut self.hits, HitRegistry::new());
        let mut scene = core::mem::replace(&mut self.scene, Scene::new());
        let mut widgets = WidgetArena::new();
        let mut entries = crate::view::entries();
        let count = view::build(
            self,
            theme,
            &mut cache,
            &mut widgets,
            &mut entries,
            viewport,
        )
        .map_err(|_| RenderError::Build)?;
        ui::collect_hits(&widgets, &entries[..count], &mut hits).map_err(|_| RenderError::Build)?;

        let mut frame = SceneArena::new();
        for entry in &entries[..count] {
            let Some(widget) = widgets.find(entry.id) else {
                continue;
            };
            frame
                .push(SceneNode::new(entry.id, entry.rect, widget.fingerprint()))
                .map_err(RenderError::Scene)?;
        }
        let damage = scene.diff(&frame, viewport);
        paint_damage_shaped(
            surface,
            &damage,
            theme.tones.background,
            Some(theme.shape),
            |painter, _| paint_widgets(painter, &widgets, &entries[..count], theme),
        );
        self.scene = scene;
        self.hits = hits;
        self.cache = Some(cache);
        Ok(damage)
    }

    fn push(&mut self, intent: LabIntent) {
        if self.intent_len < INTENT_CAPACITY {
            self.intents[self.intent_len] = Some(intent);
            self.intent_len += 1;
        }
    }

    pub(super) fn activate(&mut self, id: u16) {
        match id {
            view::RECORD_HIT_ID => self.toggle_record(0),
            view::TONE_TEST_HIT_ID => self.toggle_record(1_000),
            view::PLAY_HIT_ID => self.push(LabIntent::Command(LabCommand::PlayClip {
                channel: self.channel,
            })),
            view::TONE_1K_HIT_ID => self.play_tone(1_000),
            view::TONE_440_HIT_ID => self.play_tone(440),
            view::DURATION_HIT_ID => self.record_ms = next_duration(self.record_ms),
            view::GAIN_HIT_ID => {
                self.gain_db = next_gain(self.gain_db);
                self.push(LabIntent::SetMicGain(self.gain_db));
            }
            view::CHANNEL_HIT_ID => self.channel = self.channel.next(),
            view::VOLUME_HIT_ID => {
                self.volume = next_volume(self.volume);
                self.push(LabIntent::SetVolume(self.volume));
            }
            view::MUTE_HIT_ID => {
                self.muted = !self.muted;
                self.push(LabIntent::SetMute(self.muted));
            }
            _ => return,
        }
        self.refresh_text();
    }

    fn select_tab(&mut self, x: u16) {
        let Some(target) = self
            .hits
            .targets()
            .iter()
            .find(|target| target.id == view::TABS_HIT_ID)
        else {
            return;
        };
        let width = target.rect.width.max(1);
        let offset = x.saturating_sub(target.rect.x);
        self.page = match (u32::from(offset) * 3 / u32::from(width)).min(2) {
            0 => Page::Loopback,
            1 => Page::Mic,
            _ => Page::Speaker,
        };
    }

    fn toggle_record(&mut self, tone_hz: u16) {
        if self.is_recording {
            self.push(LabIntent::Command(LabCommand::Stop));
        } else {
            self.push(LabIntent::Command(LabCommand::Record {
                duration_ms: self.record_ms,
                tone_hz,
            }));
        }
    }

    fn play_tone(&mut self, frequency_hz: u16) {
        self.push(LabIntent::Command(LabCommand::PlayTone {
            frequency_hz,
            duration_ms: 1_000,
        }));
    }

    fn set_mode(&mut self, mode: Mode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        if mode != Mode::Desk {
            self.is_recording = false;
            self.is_playing = false;
        }
        self.scene.request_full(RedrawReason::ModeSwitch);
    }

    fn refresh_text(&mut self) {
        self.status_len = format::status_text(
            self.is_recording,
            self.is_playing,
            self.elapsed_ms,
            &mut self.status_text,
        )
        .len();
        self.mic1_len = format::mic_text("MIC1 ", self.mic1, &mut self.mic1_text).len();
        self.mic2_len = format::mic_text("MIC2 ", self.mic2, &mut self.mic2_text).len();
        self.gain_len = format::gain_text(self.gain_db, &mut self.gain_text).len();
        self.volume_len = format::percent_text(self.volume, &mut self.volume_text).len();
        self.duration_len = format::duration_text(self.record_ms, &mut self.duration_text).len();
        let report = self.report.unwrap_or_default();
        self.rate_len = format::rate_text(report, &mut self.rate_text).len();
        self.hz1_len = format::hz_text(report.mic1_hz, &mut self.hz1_text).len();
        self.hz2_len = format::hz_text(report.mic2_hz, &mut self.hz2_text).len();
        self.frames_len = format::count_text(report.frames, &mut self.frames_text).len();
        self.dc1_len = format::dc_text(report.mic1_dc, &mut self.dc1_text).len();
        self.dc2_len = format::dc_text(report.mic2_dc, &mut self.dc2_text).len();
        self.detail_len = format::detail_text(
            self.page.index(),
            text(&self.rate_text[..self.rate_len]),
            text(&self.hz1_text[..self.hz1_len]),
            text(&self.hz2_text[..self.hz2_len]),
            text(&self.dc1_text[..self.dc1_len]),
            text(&self.dc2_text[..self.dc2_len]),
            &mut self.detail_text,
        )
        .len();
        self.error_len = match self.error {
            Some(error) => {
                let label = error.label().as_bytes();
                let len = label.len().min(self.error_text.len());
                self.error_text[..len].copy_from_slice(&label[..len]);
                len
            }
            None => 0,
        };
    }
}

impl App for AudioLab {
    fn enter(&mut self, context: &mut AppContext<'_>) -> Result<(), AppError> {
        self.set_mode(context.mode());
        Ok(())
    }

    fn exit(&mut self, _context: &mut AppContext<'_>) {
        self.push(LabIntent::Command(LabCommand::Stop));
        self.is_recording = false;
        self.is_playing = false;
    }

    fn tick(&mut self, context: &mut AppContext<'_>) {
        self.set_mode(context.mode());
    }

    fn tap(&mut self, x: u16, y: u16, _context: &mut AppContext<'_>) -> Result<(), AppError> {
        if self.mode != Mode::Desk {
            return Ok(());
        }
        if let Some(id) = self.hits.hit(x, y) {
            if id == view::TABS_HIT_ID {
                self.select_tab(x);
            } else {
                self.activate(id);
            }
        }
        Ok(())
    }

    fn invalidate(&mut self) {
        self.scene.request_full(RedrawReason::Overlay);
    }

    fn render(
        &mut self,
        context: &AppContext<'_>,
        theme: &Theme,
        surface: &mut dyn Surface,
    ) -> Result<Damage, AppError> {
        self.set_mode(context.mode());
        self.render_frame(surface, theme, context.viewport())
            .map_err(|_| AppError::Failed(RENDER_FAILED_CODE))
    }
}

fn text(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes).unwrap_or("")
}

fn next_duration(current: u16) -> u16 {
    let index = DURATIONS_MS
        .iter()
        .position(|&value| value == current)
        .unwrap_or(1);
    DURATIONS_MS[(index + 1) % DURATIONS_MS.len()]
}

pub(super) fn next_gain(current: u8) -> u8 {
    let next = current.saturating_add(GAIN_STEP_DB);
    if next > MAX_GAIN_DB {
        0
    } else {
        next
    }
}

pub(super) fn next_volume(current: u8) -> u8 {
    if current >= 100 {
        0
    } else {
        (current / 10 + 1) * 10
    }
}
