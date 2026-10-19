//! The real Home app: lifecycle, state updates, and the frame it paints.

use common::{Mode, Percent, Region, Timestamp};
use settings::TimeFormat;
use shell::{App, AppContext, AppError};
use ui::{
    paint_damage_shaped, paint_widgets, Damage, RedrawReason, Scene, SceneArena, SceneError,
    SceneNode, Surface, TextCache, Theme, WidgetArena,
};

use super::{build, HomeState};
use crate::view;

const RENDER_FAILED_CODE: u16 = 1;

/// A Home frame failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// A screen could not be built or laid out.
    Build,
    /// The scene arena overflowed.
    Scene(SceneError),
}

/// Home for the active runtime mode.
pub struct Home {
    mode: Mode,
    state: HomeState,
    cache: TextCache,
    scene: Scene,
}

impl Home {
    /// Creates Home with a settings-derived clock format.
    pub fn new(
        mode: Mode,
        now: Timestamp,
        battery: Percent,
        is_charging: bool,
        time_format: TimeFormat,
    ) -> Self {
        Self {
            mode,
            state: HomeState::new_with_format(now, battery, is_charging, time_format),
            cache: TextCache::new(),
            scene: Scene::new(),
        }
    }

    /// The active mode.
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// Updates the displayed values.
    pub fn update(&mut self, now: Timestamp, battery: Percent, is_charging: bool) -> bool {
        self.state.update(now, battery, is_charging)
    }

    /// Updates the clock format.
    pub fn set_time_format(&mut self, time_format: TimeFormat) -> bool {
        self.state.set_time_format(time_format)
    }

    /// Sets or clears the ringing alarm.
    pub fn set_alarm(&mut self, alarm: Option<Timestamp>) -> bool {
        self.state.set_alarm(alarm)
    }

    /// Builds, diffs, and paints the Home frame.
    pub fn render_frame(
        &mut self,
        surface: &mut dyn Surface,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, RenderError> {
        let Self {
            mode,
            state,
            cache,
            scene,
        } = self;
        let mut widgets = WidgetArena::new();
        let mut entries = view::entries();
        let count = {
            let mut builder = view::Builder {
                theme,
                cache,
                widgets: &mut widgets,
                entries: &mut entries,
                viewport,
            };
            build(state, *mode, &mut builder).map_err(|_| RenderError::Build)?
        };

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

    fn sync_context(&mut self, context: &AppContext<'_>) {
        self.set_mode(context.mode());
        let status = context.status();
        self.update(context.now(), status.battery(), status.is_charging());
    }

    fn set_mode(&mut self, mode: Mode) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.scene.request_full(RedrawReason::ModeSwitch);
    }
}

impl App for Home {
    fn enter(&mut self, context: &mut AppContext<'_>) -> Result<(), AppError> {
        self.sync_context(context);
        Ok(())
    }

    fn tick(&mut self, context: &mut AppContext<'_>) {
        self.sync_context(context);
    }

    fn render(
        &mut self,
        context: &AppContext<'_>,
        theme: &Theme,
        surface: &mut dyn Surface,
    ) -> Result<Damage, AppError> {
        self.sync_context(context);
        self.render_frame(surface, theme, context.viewport())
            .map_err(|_| AppError::Failed(RENDER_FAILED_CODE))
    }
}
