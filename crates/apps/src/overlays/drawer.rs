//! The app drawer: a full-page, one-column grid on watch.

use common::Region;
use shell::{Action, App, AppContext, AppError};
use ui::{
    collect_hits, paint_damage_shaped, paint_widgets, Align, Damage, HitRegistry, IconId,
    LabelProps, Length, RedrawReason, Scene, SceneArena, SceneNode, Surface, TextAlign, TextCache,
    TextRole, Theme, TileProps, Widget, WidgetArena, WidgetKind,
};

use crate::view::{
    background, column, content_padding, crown, fill_node, grid, BuildError, Builder,
};

const RENDER_FAILED_CODE: u16 = 1;

const BG_ID: u16 = 850;
const TITLE_ID: u16 = 851;
const GRID_ID: u16 = 852;
const CROWN_ID: u16 = 853;
const CONTENT_ID: u16 = 854;
const BODY_ID: u16 = 855;
const TILE_BASE: u16 = 860;

/// Settings tile.
pub const SETTINGS_ID: u16 = TILE_BASE;

/// The drawer with one tile.
pub struct Drawer {
    columns: u8,
    hits: HitRegistry,
    cache: TextCache,
    scene: Scene,
}

impl Drawer {
    /// Creates the drawer with a future mode-specific column count.
    pub fn new(columns: u8) -> Self {
        Self {
            columns: columns.max(1),
            hits: HitRegistry::new(),
            cache: TextCache::new(),
            scene: Scene::new(),
        }
    }

    /// Builds, diffs, and paints the drawer frame.
    pub fn render_frame(
        &mut self,
        surface: &mut dyn Surface,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, AppError> {
        let Self {
            columns,
            hits,
            cache,
            scene,
        } = self;
        let mut widgets = WidgetArena::new();
        let mut entries = crate::view::entries();
        let count = build(*columns, theme, cache, &mut widgets, &mut entries, viewport)
            .map_err(|_| AppError::Failed(RENDER_FAILED_CODE))?;
        collect_hits(&widgets, &entries[..count], hits)
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

impl App for Drawer {
    fn tap(&mut self, x: u16, y: u16, context: &mut AppContext<'_>) -> Result<(), AppError> {
        let action = match self.hits.hit(x, y) {
            Some(SETTINGS_ID) => Action::OpenApp(crate::SETTINGS_APP_ID),
            _ => Action::CloseAppDrawer,
        };
        context
            .push_action(action)
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
    columns: u8,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let title = title();
    let settings = Widget::new(
        SETTINGS_ID,
        WidgetKind::Tile(TileProps {
            icon: IconId::GearSix,
            label: "Settings",
        }),
    );
    let title_node = fill_node(&title, theme, cache, viewport.width);
    let tile_height = theme.min_touch_px.saturating_mul(2);
    let tile_nodes = [crate::view::node(
        &settings,
        theme,
        cache,
        Length::Fill(1),
        tile_height,
    )];

    let mut builder = Builder {
        theme,
        cache,
        widgets,
        entries,
        viewport,
    };
    builder.push(background(BG_ID))?;
    builder.push(title)?;
    builder.push(settings)?;

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let apps = grid(
        GRID_ID,
        &tile_nodes,
        columns,
        theme.spacing.sm,
        theme.spacing.sm,
    );
    let content_children = [
        crate::view::spacer(856, Length::Fill(1), Length::Fill(1)),
        apps,
        crate::view::spacer(857, Length::Fill(1), Length::Fill(1)),
    ];
    let content = column(
        CONTENT_ID,
        &content_children,
        theme.spacing.sm,
        Align::Start,
    )
    .with_padding(content_padding(theme));
    let body_children = [crown_node, content];
    let body = column(BODY_ID, &body_children, 0, Align::Start);
    let root_children = [
        crate::view::leaf(BG_ID, Length::Fill(1), Length::Fill(1)),
        body,
    ];
    let root = crate::view::stack(0, &root_children, Align::Start);
    builder.layout(&root)
}

fn title<'a>() -> Widget<'a> {
    Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Apps",
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::{Mode, Region, TouchPoint, TouchReport};
    use shell::Shell;
    use ui::{MonoSurface, Theme};

    struct TestRegistry {
        drawer: Drawer,
    }

    impl shell::AppRegistry for TestRegistry {
        fn with_app<R>(
            &mut self,
            _id: shell::AppId,
            _f: impl FnOnce(&mut dyn App) -> R,
        ) -> Option<R> {
            None
        }

        fn with_layer<R>(
            &mut self,
            kind: shell::LayerKind,
            f: impl FnOnce(&mut dyn App) -> R,
        ) -> Option<R> {
            match kind {
                shell::LayerKind::AppDrawer => Some(f(&mut self.drawer)),
                _ => None,
            }
        }
    }

    #[test]
    fn drawer_renders_one_tile_and_shell_routes_a_tap() {
        let mut data = vec![0u8; MonoSurface::buffer_len(410, 502)];
        let mut surface = MonoSurface::new(410, 502, &mut data).unwrap();
        let mut drawer = Drawer::new(1);
        let damage = drawer
            .render_frame(&mut surface, &Theme::WATCH, Region::full(410, 502))
            .unwrap();
        assert!(damage.is_full());
        let tile = drawer
            .hits
            .targets()
            .iter()
            .find(|target| target.id == SETTINGS_ID)
            .copied()
            .unwrap();
        let x = tile.rect.x + tile.rect.width / 2;
        let y = tile.rect.y + tile.rect.height / 2;

        let registry = TestRegistry { drawer };
        let mut shell = Shell::new(
            registry,
            shell::AppId::new(1),
            Mode::Watch,
            Region::full(410, 502),
        );
        shell.dispatch(Action::OpenAppDrawer).unwrap();
        let start = TouchReport::new(&[TouchPoint::new(1, x, y)]);
        shell.handle_touch(&start, 0).unwrap();
        shell.handle_touch(&TouchReport::empty(), 100).unwrap();
        assert_eq!(shell.active_app(), Some(crate::SETTINGS_APP_ID));
    }
}
