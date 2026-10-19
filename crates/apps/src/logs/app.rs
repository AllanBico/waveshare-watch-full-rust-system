//! Logs app state, data loading, lifecycle, and paging.

use common::{LogEntry, Region};
use shell::{App, AppContext, AppError, Swipe, SwipeDirection, SwipeOrigin};
use ui::{
    collect_hits, paint_damage_shaped, paint_widgets, Damage, HitRegistry, RedrawReason, Scene,
    SceneArena, SceneError, SceneNode, Surface, TextCache, Theme, WidgetArena,
};

use super::format;
use super::model::{LogCategory, LogPage, LOGS_PER_PAGE};
use super::view;

const RENDER_FAILED_CODE: u16 = 1;

/// The screen currently shown by the Logs app.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Categories,
    Category(LogCategory),
}

/// A Logs frame failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// A screen could not be built or laid out.
    Build,
    /// The scene arena overflowed.
    Scene(SceneError),
}

/// The persistent log viewer.
///
/// Only one category is held in memory at a time. Tapping a category queues a
/// load request that the firmware fulfills before the next frame.
pub struct LogsApp {
    view: View,
    entries: LogPage,
    load_request: Option<LogCategory>,
    page: usize,
    event_texts: [[u8; 28]; LOGS_PER_PAGE],
    time_texts: [[u8; 12]; LOGS_PER_PAGE],
    hits: HitRegistry,
    cache: TextCache,
    scene: Scene,
}

impl LogsApp {
    /// Creates an empty log viewer on the category screen.
    pub fn new() -> Self {
        Self {
            view: View::Categories,
            entries: LogPage::new(),
            load_request: None,
            page: 0,
            event_texts: [[0; 28]; LOGS_PER_PAGE],
            time_texts: [[0; 12]; LOGS_PER_PAGE],
            hits: HitRegistry::new(),
            cache: TextCache::new(),
            scene: Scene::new(),
        }
    }

    /// Starts a fresh load for one category.
    pub fn begin_load(&mut self, category: LogCategory) {
        self.entries.clear();
        self.load_request = None;
        self.view = View::Category(category);
        self.page = 0;
    }

    /// Adds one entry when it belongs to the category being loaded.
    pub fn ingest(&mut self, entry: LogEntry) {
        let View::Category(category) = self.view else {
            return;
        };
        if category.matches(entry.event) {
            self.entries.push_unique(entry);
        }
    }

    /// Sorts the loaded category after a load.
    pub fn finish_load(&mut self) {
        self.entries.sort_newest_first();
    }

    /// Whether the loaded category is full, so older entries are useless.
    pub fn is_full(&self) -> bool {
        self.entries.is_full()
    }

    /// Takes a pending category load request.
    pub fn take_load_request(&mut self) -> Option<LogCategory> {
        self.load_request.take()
    }

    /// Builds, diffs, and paints the current screen.
    pub fn render_frame(
        &mut self,
        surface: &mut dyn Surface,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, RenderError> {
        self.refresh_rows();
        let Self {
            view,
            entries,
            event_texts,
            time_texts,
            hits,
            cache,
            scene,
            page,
            ..
        } = self;
        let mut widgets = WidgetArena::new();
        let mut entries_out = crate::view::entries();
        let count = match *view {
            View::Categories => {
                view::build_categories(theme, cache, &mut widgets, &mut entries_out, viewport)
            }
            View::Category(category) => view::build_list(
                category,
                entries.entries(),
                *page,
                event_texts,
                time_texts,
                theme,
                cache,
                &mut widgets,
                &mut entries_out,
                viewport,
            ),
        }
        .map_err(|_| RenderError::Build)?;
        collect_hits(&widgets, &entries_out[..count], hits).map_err(|_| RenderError::Build)?;

        let mut frame = SceneArena::new();
        for entry in &entries_out[..count] {
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
            |painter, _| paint_widgets(painter, &widgets, &entries_out[..count], theme),
        );
        Ok(damage)
    }

    fn refresh_rows(&mut self) {
        let View::Category(_) = self.view else {
            return;
        };
        let entries = self.entries.entries();
        let start = self.page.saturating_mul(LOGS_PER_PAGE);
        for index in 0..LOGS_PER_PAGE {
            self.event_texts[index] = [0; 28];
            self.time_texts[index] = [0; 12];
            if let Some(entry) = entries.get(start + index) {
                format::event_text(entry, &mut self.event_texts[index]);
                format::time_text(entry.timestamp, &mut self.time_texts[index]);
            }
        }
    }

    fn page_count(&self) -> usize {
        self.entries.entries().len().div_ceil(LOGS_PER_PAGE).max(1)
    }

    fn open_category(&mut self, category: LogCategory) {
        self.view = View::Category(category);
        self.entries.clear();
        self.page = 0;
        self.load_request = Some(category);
    }

    fn older_page(&mut self) -> bool {
        if !matches!(self.view, View::Category(_)) {
            return false;
        }
        let next = self.page.saturating_add(1);
        if next >= self.page_count() {
            return false;
        }
        self.page = next;
        true
    }

    fn newer_page(&mut self) -> bool {
        if self.page == 0 {
            return false;
        }
        self.page -= 1;
        true
    }

    fn close_category(&mut self) -> bool {
        if matches!(self.view, View::Categories) {
            return false;
        }
        self.view = View::Categories;
        self.load_request = None;
        self.page = 0;
        self.entries.clear();
        true
    }
}

impl Default for LogsApp {
    fn default() -> Self {
        Self::new()
    }
}

impl App for LogsApp {
    fn enter(&mut self, _context: &mut AppContext<'_>) -> Result<(), AppError> {
        self.view = View::Categories;
        self.load_request = None;
        self.page = 0;
        Ok(())
    }

    fn tap(&mut self, x: u16, y: u16, _context: &mut AppContext<'_>) -> Result<(), AppError> {
        if !matches!(self.view, View::Categories) {
            return Ok(());
        }
        let Some(id) = self.hits.hit(x, y) else {
            return Ok(());
        };
        let Some(index) = view::category_index(id) else {
            return Ok(());
        };
        self.open_category(LogCategory::ALL[index]);
        Ok(())
    }

    fn swipe(&mut self, swipe: Swipe, _context: &mut AppContext<'_>) -> Result<bool, AppError> {
        if !matches!(self.view, View::Category(_)) || swipe.origin != SwipeOrigin::Content {
            return Ok(false);
        }
        Ok(match swipe.direction {
            SwipeDirection::Up => self.older_page(),
            SwipeDirection::Down => self.newer_page(),
            _ => false,
        })
    }

    fn back(&mut self, _context: &mut AppContext<'_>) -> Result<bool, AppError> {
        Ok(self.close_category())
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
            .map_err(|_| AppError::Failed(RENDER_FAILED_CODE))
    }
}

#[cfg(test)]
impl LogsApp {
    pub(crate) fn count_for_test(&self) -> usize {
        self.entries.entries().len()
    }

    pub(crate) fn entry_for_test(&self, index: usize) -> Option<LogEntry> {
        self.entries.entries().get(index).copied()
    }

    pub(crate) fn page_for_test(&self) -> usize {
        self.page
    }

    pub(crate) fn page_count_for_test(&self) -> usize {
        self.page_count()
    }

    pub(crate) fn older_for_test(&mut self) -> bool {
        self.older_page()
    }

    pub(crate) fn newer_for_test(&mut self) -> bool {
        self.newer_page()
    }

    pub(crate) fn back_for_test(&mut self) -> bool {
        self.close_category()
    }
}
