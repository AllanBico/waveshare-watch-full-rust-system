//! Frame building, damage diffing, and painting for the preview.

use common::Region;
use ui::{
    paint_damage_shaped, paint_widgets, Damage, IconId, LayoutRect, SceneArena, SceneError,
    SceneNode, ScreenShape, Surface, Theme, WidgetArena, WidgetError,
};

use super::{
    calibrate, chrome, drawer, gallery, placeholder, quick, settings_screen, Overlay, Preview,
    Screen,
};
use crate::view::{self, BuildError};

/// A preview frame failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// A screen could not be built.
    Build,
    /// Hit registration failed.
    Widget(WidgetError),
    /// The scene arena overflowed.
    Scene(SceneError),
}

impl From<BuildError> for RenderError {
    fn from(_: BuildError) -> Self {
        Self::Build
    }
}

impl From<WidgetError> for RenderError {
    fn from(error: WidgetError) -> Self {
        Self::Widget(error)
    }
}

impl From<SceneError> for RenderError {
    fn from(error: SceneError) -> Self {
        Self::Scene(error)
    }
}

impl Preview {
    /// Builds, diffs, and paints one frame. Returns the damage to flush.
    pub fn render<S: Surface>(
        &mut self,
        surface: &mut S,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, RenderError> {
        let Self {
            mode,
            screen,
            overlay,
            home,
            settings,
            calibrate,
            gallery_page,
            settings_offset,
            brightness_text,
            brightness_len,
            volume_text,
            volume_len,
            hits,
            scene,
            cache,
            is_dirty,
            ..
        } = self;
        let is_calibrate = *screen == Screen::Calibrate && *overlay == Overlay::None;
        let mut widgets = WidgetArena::new();
        let mut base_entries = view::entries();
        let mut overlay_entries = view::entries();
        let mut merged = view::merged_entries();

        let base_count = {
            let mut builder = view::Builder {
                theme,
                cache,
                widgets: &mut widgets,
                entries: &mut base_entries,
                viewport,
            };
            build_base(
                BaseInput {
                    screen: *screen,
                    mode: *mode,
                    home,
                    calibrate,
                    settings,
                    brightness: text(brightness_text, *brightness_len),
                    volume: text(volume_text, *volume_len),
                    settings_offset: *settings_offset,
                    gallery_page: *gallery_page,
                },
                &mut builder,
            )?
        };
        let overlay_count = {
            let mut builder = view::Builder {
                theme,
                cache,
                widgets: &mut widgets,
                entries: &mut overlay_entries,
                viewport,
            };
            build_overlay(*overlay, settings, *mode, &mut builder)?
        };
        let count = view::merge(
            &base_entries[..base_count],
            &overlay_entries[..overlay_count],
            &mut merged,
        );
        ui::collect_hits(&widgets, &merged[..count], hits)?;
        let calibration = is_calibrate.then_some(&*calibrate);
        let frame = build_scene(&widgets, &merged[..count], calibration, viewport)?;
        let damage = scene.diff(&frame, viewport);
        let clip = if is_calibrate {
            ScreenShape::NONE
        } else {
            theme.shape
        };
        paint_damage_shaped(
            surface,
            &damage,
            theme.tones.background,
            Some(clip),
            |painter, _| {
                paint_widgets(painter, &widgets, &merged[..count], theme);
                if is_calibrate {
                    calibrate::paint(painter, calibrate, theme, viewport);
                } else {
                    chrome::paint(painter, theme, *screen, *overlay, viewport);
                }
            },
        );
        *is_dirty = false;
        Ok(damage)
    }
}

fn build_scene(
    widgets: &WidgetArena<'_>,
    entries: &[ui::LayoutEntry],
    calibrate: Option<&calibrate::CalibrateState>,
    viewport: Region,
) -> Result<SceneArena, RenderError> {
    let mut frame = build_frame(widgets, entries)?;
    if let Some(state) = calibrate {
        frame.push(SceneNode::new(
            calibrate::SCENE_ID,
            LayoutRect::from_region(viewport),
            state.fingerprint(),
        ))?;
    }
    Ok(frame)
}

fn build_frame(
    widgets: &WidgetArena<'_>,
    entries: &[ui::LayoutEntry],
) -> Result<SceneArena, SceneError> {
    let mut frame = SceneArena::new();
    for entry in entries {
        if let Some(widget) = widgets.find(entry.id) {
            frame.push(SceneNode::new(entry.id, entry.rect, widget.fingerprint()))?;
        }
    }
    Ok(frame)
}

struct BaseInput<'a> {
    screen: Screen,
    mode: common::Mode,
    home: &'a crate::home::HomeState,
    calibrate: &'a calibrate::CalibrateState,
    settings: &'a settings::Settings,
    brightness: &'a str,
    volume: &'a str,
    settings_offset: u16,
    gallery_page: u8,
}

fn build_base<'a>(
    input: BaseInput<'a>,
    view: &mut view::Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    match input.screen {
        Screen::Home => crate::home::build(input.home, input.mode, view),
        Screen::Gallery => gallery::build(input.gallery_page, view),
        Screen::Settings => settings_screen::build(
            input.settings,
            input.brightness,
            input.volume,
            input.settings_offset,
            view,
        ),
        Screen::Alarms => placeholder::build("Alarms", IconId::Bell, view),
        Screen::Calibrate => calibrate::build(input.calibrate, view),
    }
}

fn build_overlay<'a>(
    overlay: super::Overlay,
    settings: &settings::Settings,
    mode: common::Mode,
    view: &mut view::Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    match overlay {
        super::Overlay::None => Ok(0),
        super::Overlay::Drawer => drawer::build(view),
        super::Overlay::QuickSettings => quick::build(settings, mode, view),
    }
}

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}
