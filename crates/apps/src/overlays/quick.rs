//! The blank quick-settings sheet.

use common::Region;
use shell::{Action, App, AppContext, AppError};
use ui::{
    paint_damage_shaped, paint_widgets, Damage, Length, RedrawReason, Scene, SceneArena, SceneNode,
    Surface, TextCache, Theme, WidgetArena,
};

use crate::view::{background, BuildError, Builder};

const RENDER_FAILED_CODE: u16 = 1;
const BG_ID: u16 = 900;

/// Quick settings, intentionally empty until its controls are wired.
pub struct QuickSettings {
    cache: TextCache,
    scene: Scene,
}

impl QuickSettings {
    /// Creates the blank quick-settings sheet.
    pub const fn new() -> Self {
        Self {
            cache: TextCache::new(),
            scene: Scene::new(),
        }
    }

    /// Builds, diffs, and paints the sheet.
    pub fn render_frame(
        &mut self,
        surface: &mut dyn Surface,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, AppError> {
        let Self { cache, scene } = self;
        let mut widgets = WidgetArena::new();
        let mut entries = crate::view::entries();
        let count = build(theme, cache, &mut widgets, &mut entries, viewport)
            .map_err(|_| AppError::Failed(RENDER_FAILED_CODE))?;
        let mut frame = SceneArena::new();
        for entry in &entries[..count] {
            let Some(widget) = widgets.find(entry.id) else {
                continue;
            };
            frame
                .push(SceneNode::new(entry.id, entry.rect, widget.fingerprint()))
                .map_err(|_| AppError::Failed(RENDER_FAILED_CODE))?;
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
}

impl Default for QuickSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl App for QuickSettings {
    fn tap(&mut self, _x: u16, _y: u16, context: &mut AppContext<'_>) -> Result<(), AppError> {
        context
            .push_action(Action::CloseQuickSettings)
            .map_err(|_| AppError::Failed(RENDER_FAILED_CODE))
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
        self.render_frame(surface, theme, context.viewport())
    }
}

fn build<'a>(
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let mut builder = Builder {
        theme,
        cache,
        widgets,
        entries,
        viewport,
    };
    builder.push(background(BG_ID))?;
    let root_children = [crate::view::leaf(BG_ID, Length::Fill(1), Length::Fill(1))];
    let root = crate::view::stack(0, &root_children, ui::Align::Start);
    builder.layout(&root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::Region;
    use ui::{MonoSurface, Theme};

    #[test]
    fn blank_sheet_fills_and_then_settles() {
        let mut quick = QuickSettings::new();
        let mut data = vec![0u8; MonoSurface::buffer_len(410, 502)];
        let mut surface = MonoSurface::new(410, 502, &mut data).unwrap();
        let first = quick
            .render_frame(&mut surface, &Theme::WATCH, Region::full(410, 502))
            .unwrap();
        assert!(first.is_full());
        let second = quick
            .render_frame(&mut surface, &Theme::WATCH, Region::full(410, 502))
            .unwrap();
        assert!(second.is_empty());
    }
}
