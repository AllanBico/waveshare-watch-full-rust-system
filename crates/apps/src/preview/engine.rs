//! Frame building, damage diffing, and painting for the preview.

use common::Region;
use ui::{
    paint_damage, paint_widgets, Damage, IconId, SceneArena, SceneError, SceneNode, Surface, Theme,
    WidgetArena, WidgetError,
};

use super::{drawer, gallery, placeholder, quick, Preview, Screen};
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
            quick,
            gallery_page,
            hits,
            scene,
            cache,
            is_dirty,
        } = self;
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
            build_base(*screen, *mode, home, *gallery_page, &mut builder)?
        };
        let overlay_count = {
            let mut builder = view::Builder {
                theme,
                cache,
                widgets: &mut widgets,
                entries: &mut overlay_entries,
                viewport,
            };
            build_overlay(*overlay, quick, &mut builder)?
        };
        let count = view::merge(
            &base_entries[..base_count],
            &overlay_entries[..overlay_count],
            &mut merged,
        );
        ui::collect_hits(&widgets, &merged[..count], hits)?;
        let frame = build_frame(&widgets, &merged[..count])?;
        let damage = scene.diff(&frame, viewport);
        paint_damage(surface, &damage, theme.tones.background, |painter, _| {
            paint_widgets(painter, &widgets, &merged[..count], theme);
        });
        *is_dirty = false;
        Ok(damage)
    }
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

fn build_base<'a>(
    screen: Screen,
    mode: common::Mode,
    home: &'a crate::home::HomeState,
    gallery_page: u8,
    view: &mut view::Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    match screen {
        Screen::Home => crate::home::build(home, mode, view),
        Screen::Gallery => gallery::build(gallery_page, view),
        Screen::Settings => placeholder::build("Settings", IconId::GearSix, view),
        Screen::Alarms => placeholder::build("Alarms", IconId::Bell, view),
    }
}

fn build_overlay<'a>(
    overlay: super::Overlay,
    quick: &crate::preview::QuickState,
    view: &mut view::Builder<'_, 'a>,
) -> Result<usize, BuildError> {
    match overlay {
        super::Overlay::None => Ok(0),
        super::Overlay::Drawer => drawer::build(view),
        super::Overlay::QuickSettings => quick::build(quick, view),
    }
}
