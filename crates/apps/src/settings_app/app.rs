//! Settings app state, frame building, and lifecycle.

use common::{Mode, Region};
use settings::{BootPolicy, SettingId, SettingValue, Settings};
use shell::{App, AppContext, AppError};
use ui::{
    collect_hits, paint_damage_shaped, paint_widgets, Damage, HitRegistry, LabelProps, Length,
    RedrawReason, Scene, SceneArena, SceneError, SceneNode, Surface, TextAlign, TextCache,
    TextRole, Theme, Widget, WidgetArena, WidgetKind,
};

use super::rows::{next_value, rows, seconds_text};
use crate::view::{background, column, content_padding, crown, fill_node, BuildError, Builder};

const RENDER_FAILED_CODE: u16 = 1;
const INTENT_CAPACITY: usize = 8;

const BG_ID: u16 = 800;
const TITLE_ID: u16 = 802;
const CROWN_ID: u16 = 806;
const CONTENT_ID: u16 = 807;
const BODY_ID: u16 = 810;

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

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::super::rows::{BRIGHTNESS_ID, DATE_FORMAT_ID, SLEEP_TIMEOUT_ID, TIME_FORMAT_ID};
    use super::*;
    use common::{Percent, Region};
    use settings::{DateFormat, TimeFormat};
    use ui::{MonoSurface, Theme};

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
    fn sleep_timeout_is_queued() {
        let mut app = SettingsApp::new(Mode::Watch, Settings::new());
        assert!(app.activate(SLEEP_TIMEOUT_ID));
        assert_eq!(
            app.pop_intent(),
            Some((SettingId::PowerSleepTimeout, SettingValue::Seconds(60)))
        );
    }

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
