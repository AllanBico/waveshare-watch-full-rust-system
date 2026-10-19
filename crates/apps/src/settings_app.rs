//! The persisted settings screen.
//!
//! Watch settings expose only the values the firmware actually applies. The
//! app keeps a snapshot, applies edits locally for immediate feedback, and
//! queues one intent per change for the firmware-owned store.

use common::{Mode, Percent, Region};
use settings::{BootPolicy, SettingId, SettingValue, Settings};
use shell::{App, AppContext, AppError};
use ui::{
    collect_hits, paint_damage_shaped, paint_widgets, Damage, HitRegistry, IconId, LabelProps,
    Length, RedrawReason, Scene, SceneArena, SceneError, SceneNode, SettingsRowProps,
    SettingsTrailing, Surface, TextAlign, TextCache, TextRole, Theme, Widget, WidgetArena,
    WidgetKind,
};

use crate::view::{background, column, content_padding, crown, fill_node, BuildError, Builder};

const RENDER_FAILED_CODE: u16 = 1;
const INTENT_CAPACITY: usize = 8;

const BG_ID: u16 = 800;
const TITLE_ID: u16 = 802;
const CROWN_ID: u16 = 806;
const CONTENT_ID: u16 = 807;
const BODY_ID: u16 = 810;
const ROW_BASE: u16 = 820;

/// Time format row.
pub const TIME_FORMAT_ID: u16 = ROW_BASE;
/// Date format row.
pub const DATE_FORMAT_ID: u16 = ROW_BASE + 1;
/// Brightness row.
pub const BRIGHTNESS_ID: u16 = ROW_BASE + 2;
/// Sleep timeout row.
pub const SLEEP_TIMEOUT_ID: u16 = ROW_BASE + 3;

const SLEEP_STEPS: [u16; 6] = [5, 10, 15, 30, 60, 120];

/// A Settings frame failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// A screen could not be built or laid out.
    Build,
    /// The scene arena overflowed.
    Scene(SceneError),
}

/// The Settings app.
pub struct SettingsApp {
    mode: Mode,
    settings: Settings,
    intents: [Option<(SettingId, SettingValue)>; INTENT_CAPACITY],
    intent_len: usize,
    brightness_text: [u8; 4],
    brightness_len: usize,
    sleep_text: [u8; 8],
    sleep_len: usize,
    hits: HitRegistry,
    cache: TextCache,
    scene: Scene,
}

impl SettingsApp {
    /// Creates Settings with a mode and settings snapshot.
    pub fn new(mode: Mode, settings: Settings) -> Self {
        let mut app = Self {
            mode,
            settings,
            intents: [None; INTENT_CAPACITY],
            intent_len: 0,
            brightness_text: [0; 4],
            brightness_len: 0,
            sleep_text: [0; 8],
            sleep_len: 0,
            hits: HitRegistry::new(),
            cache: TextCache::new(),
            scene: Scene::new(),
        };
        app.refresh_text();
        app
    }

    /// The active mode.
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// Updates the active mode.
    pub fn set_mode(&mut self, mode: Mode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.scene.request_full(RedrawReason::ModeSwitch);
        self.refresh_text();
    }

    /// Replaces the snapshot after the firmware store applies changes.
    pub fn sync_settings(&mut self, settings: Settings) {
        self.settings = settings;
        self.refresh_text();
    }

    /// Removes the oldest setting queued for persistence.
    pub fn pop_intent(&mut self) -> Option<(SettingId, SettingValue)> {
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

    /// Builds, diffs, and paints the Settings frame.
    pub fn render_frame(
        &mut self,
        surface: &mut dyn Surface,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, RenderError> {
        let Self {
            settings,
            brightness_text,
            brightness_len,
            sleep_text,
            sleep_len,
            hits,
            cache,
            scene,
            ..
        } = self;
        let brightness = text(brightness_text, *brightness_len);
        let sleep = text(sleep_text, *sleep_len);
        let mut widgets = WidgetArena::new();
        let mut entries = crate::view::entries();
        let count = build(
            settings,
            brightness,
            sleep,
            theme,
            cache,
            &mut widgets,
            &mut entries,
            viewport,
        )
        .map_err(|_| RenderError::Build)?;
        collect_hits(&widgets, &entries[..count], hits).map_err(|_| RenderError::Build)?;

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
        Ok(damage)
    }

    fn refresh_text(&mut self) {
        let brightness = self.settings.brightness(self.mode);
        self.brightness_len =
            crate::format::percent_text(brightness, &mut self.brightness_text).len();
        let idle_secs = (BootPolicy::from_settings(&self.settings).idle_timeout_ms / 1_000) as u16;
        self.sleep_len = seconds_text(idle_secs, &mut self.sleep_text).len();
    }

    fn activate(&mut self, id: u16) -> bool {
        if self.intent_len >= INTENT_CAPACITY {
            return false;
        }
        let Some((setting, value)) = next_value(id, &self.settings, self.mode) else {
            return false;
        };
        if setting.apply(&mut self.settings, value).is_err() {
            return false;
        }
        if let Some(slot) = self.intents.get_mut(self.intent_len) {
            *slot = Some((setting, value));
            self.intent_len += 1;
        }
        self.refresh_text();
        true
    }
}

impl App for SettingsApp {
    fn enter(&mut self, context: &mut AppContext<'_>) -> Result<(), AppError> {
        self.set_mode(context.mode());
        Ok(())
    }

    fn tick(&mut self, context: &mut AppContext<'_>) {
        self.set_mode(context.mode());
    }

    fn tap(&mut self, x: u16, y: u16, _context: &mut AppContext<'_>) -> Result<(), AppError> {
        if let Some(id) = self.hits.hit(x, y) {
            self.activate(id);
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

fn build<'a>(
    settings: &'a Settings,
    brightness: &'a str,
    sleep: &'a str,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let title = title();
    let rows = rows(settings, brightness, sleep);
    let title_node = fill_node(&title, theme, cache, viewport.width);
    let row_nodes = rows.map(|row| fill_node(&row, theme, cache, viewport.width));

    let mut builder = Builder {
        theme,
        cache,
        widgets,
        entries,
        viewport,
    };
    builder.push(background(BG_ID))?;
    builder.push(title)?;
    for row in rows {
        builder.push(row)?;
    }

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let content = column(CONTENT_ID, &row_nodes, theme.spacing.xs, ui::Align::Start)
        .with_padding(content_padding(theme));
    let body_children = [crown_node, content];
    let body = column(BODY_ID, &body_children, 0, ui::Align::Start);
    let root_children = [
        crate::view::leaf(BG_ID, Length::Fill(1), Length::Fill(1)),
        body,
    ];
    let root = crate::view::stack(0, &root_children, ui::Align::Start);
    builder.layout(&root)
}

fn title<'a>() -> Widget<'a> {
    Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Settings",
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    )
}

fn rows<'a>(settings: &Settings, brightness: &'a str, sleep: &'a str) -> [Widget<'a>; 4] {
    [
        row(
            TIME_FORMAT_ID,
            "Time format",
            "Clock",
            IconId::Clock,
            SettingsTrailing::Value(settings.display.time_format.label()),
        ),
        row(
            DATE_FORMAT_ID,
            "Date format",
            "Calendar",
            IconId::List,
            SettingsTrailing::Value(settings.display.date_format.label()),
        ),
        row(
            BRIGHTNESS_ID,
            "Brightness",
            "Display",
            IconId::Sun,
            SettingsTrailing::Value(brightness),
        ),
        row(
            SLEEP_TIMEOUT_ID,
            "Sleep timeout",
            "Power",
            IconId::Power,
            SettingsTrailing::Value(sleep),
        ),
    ]
}

fn row<'a>(
    id: u16,
    label: &'a str,
    subtitle: &'a str,
    icon: IconId,
    trailing: SettingsTrailing<'a>,
) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::SettingsRow(SettingsRowProps {
            title: label,
            subtitle,
            icon: Some(icon),
            trailing,
        }),
    )
}

fn next_value(id: u16, settings: &Settings, mode: Mode) -> Option<(SettingId, SettingValue)> {
    let change = match id {
        TIME_FORMAT_ID => (
            SettingId::DisplayTimeFormat,
            SettingValue::TimeFormat(settings.display.time_format.next()),
        ),
        DATE_FORMAT_ID => (
            SettingId::DisplayDateFormat,
            SettingValue::DateFormat(settings.display.date_format.next()),
        ),
        BRIGHTNESS_ID => (
            SettingId::brightness(mode),
            SettingValue::Percent(next_percent(settings.brightness(mode))),
        ),
        SLEEP_TIMEOUT_ID => {
            let cap = settings.power.awake_cap_secs;
            let current = settings.power.sleep_timeout_secs.min(cap);
            (
                SettingId::PowerSleepTimeout,
                SettingValue::Seconds(next_sleep(current, cap)),
            )
        }
        _ => return None,
    };
    Some(change)
}

fn next_percent(value: Percent) -> Percent {
    Percent::new(((value.value() / 10) + 1) % 11 * 10)
}

fn next_sleep(current: u16, cap: u16) -> u16 {
    SLEEP_STEPS
        .iter()
        .copied()
        .find(|step| *step > current && *step <= cap)
        .unwrap_or(SLEEP_STEPS[0])
}

fn seconds_text(seconds: u16, out: &mut [u8; 8]) -> &str {
    let mut digits = [0u8; 5];
    let mut len = 0usize;
    let mut value = seconds;
    if value == 0 {
        digits[0] = b'0';
        len = 1;
    } else {
        while value > 0 && len < digits.len() {
            digits[len] = b'0' + (value % 10) as u8;
            value /= 10;
            len += 1;
        }
    }
    for index in 0..len {
        out[index] = digits[len - 1 - index];
    }
    out[len] = b's';
    text(out, len + 1)
}

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::Mode;
    use settings::{DateFormat, TimeFormat};

    #[test]
    fn time_and_date_toggle_live() {
        let mut app = SettingsApp::new(Mode::Watch, Settings::new());
        assert!(app.activate(TIME_FORMAT_ID));
        assert_eq!(
            app.pop_intent(),
            Some((
                SettingId::DisplayTimeFormat,
                SettingValue::TimeFormat(TimeFormat::Hour12)
            ))
        );
        assert!(app.activate(DATE_FORMAT_ID));
        assert_eq!(
            app.pop_intent(),
            Some((
                SettingId::DisplayDateFormat,
                SettingValue::DateFormat(DateFormat::DayMonth)
            ))
        );
    }

    #[test]
    fn brightness_uses_the_active_mode_and_cycles_by_ten() {
        let mut app = SettingsApp::new(Mode::Watch, Settings::new());
        assert!(app.activate(BRIGHTNESS_ID));
        assert_eq!(
            app.pop_intent(),
            Some((
                SettingId::DisplayBrightnessWatch,
                SettingValue::Percent(Percent::new(80))
            ))
        );
        app.set_mode(Mode::Desk);
        assert!(app.activate(BRIGHTNESS_ID));
        assert_eq!(
            app.pop_intent(),
            Some((
                SettingId::DisplayBrightnessDesk,
                SettingValue::Percent(Percent::new(80))
            ))
        );
    }

    #[test]
    fn sleep_timeout_advances_through_steps() {
        assert_eq!(next_sleep(30, 120), 60);
        assert_eq!(next_sleep(3_600, 120), 5);
        assert_eq!(next_sleep(7, 120), 10);
        assert_eq!(next_sleep(60, 60), 5);

        let mut app = SettingsApp::new(Mode::Watch, Settings::new());
        assert!(app.activate(SLEEP_TIMEOUT_ID));
        assert_eq!(
            app.pop_intent(),
            Some((SettingId::PowerSleepTimeout, SettingValue::Seconds(60)))
        );
    }

    #[test]
    fn seconds_text_is_short_and_bounded() {
        let mut buffer = [0u8; 8];
        assert_eq!(seconds_text(5, &mut buffer), "5s");
        assert_eq!(seconds_text(3_600, &mut buffer), "3600s");
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;
    use common::Region;
    use ui::{MonoSurface, Theme};

    #[test]
    fn settings_renders_and_registers_rows() {
        let mut app = SettingsApp::new(Mode::Watch, Settings::new());
        let mut data = vec![0u8; MonoSurface::buffer_len(410, 502)];
        let mut surface = MonoSurface::new(410, 502, &mut data).unwrap();
        let damage = app
            .render_frame(&mut surface, &Theme::WATCH, Region::full(410, 502))
            .unwrap();
        assert!(damage.is_full());
        assert!(surface.lit_pixels() > 0);
        let ids: Vec<u16> = app.hits.targets().iter().map(|target| target.id).collect();
        assert!(ids.contains(&TIME_FORMAT_ID));
        assert!(ids.contains(&DATE_FORMAT_ID));
        assert!(ids.contains(&BRIGHTNESS_ID));
        assert!(ids.contains(&SLEEP_TIMEOUT_ID));
    }
}
