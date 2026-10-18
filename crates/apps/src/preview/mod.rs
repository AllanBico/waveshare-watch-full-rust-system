//! Hardware preview screens: Home, overlays, placeholders, and the gallery.

mod drawer;
mod engine;
mod gallery;
mod placeholder;
mod quick;

#[cfg(test)]
mod tests;

pub use engine::RenderError;
pub use gallery::PAGES;
pub use quick::QuickState;

use common::{Mode, Percent, Timestamp};
use ui::{HitRegistry, Scene, TextCache};

use crate::home::HomeState;

/// The base screen currently shown by the preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Home.
    Home,
    /// Component gallery.
    Gallery,
    /// Settings placeholder.
    Settings,
    /// Alarms placeholder.
    Alarms,
}

/// The overlay currently shown above the base screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overlay {
    /// No overlay.
    None,
    /// App drawer.
    Drawer,
    /// Quick settings.
    QuickSettings,
}

/// Fixed preview state shared by the firmware and host tests.
pub struct Preview {
    mode: Mode,
    screen: Screen,
    overlay: Overlay,
    home: HomeState,
    quick: QuickState,
    gallery_page: u8,
    hits: HitRegistry,
    scene: Scene,
    cache: TextCache,
    is_dirty: bool,
}

impl Preview {
    /// Creates a preview booted to Home.
    pub fn new(mode: Mode, now: Timestamp, battery: Percent, is_charging: bool) -> Self {
        Self {
            mode,
            screen: Screen::Home,
            overlay: Overlay::None,
            home: HomeState::new(now, battery, is_charging),
            quick: QuickState::new(),
            gallery_page: 0,
            hits: HitRegistry::new(),
            scene: Scene::new(),
            cache: TextCache::new(),
            is_dirty: true,
        }
    }

    /// The active mode.
    pub const fn mode(&self) -> Mode {
        self.mode
    }

    /// The active base screen.
    pub const fn screen(&self) -> Screen {
        self.screen
    }

    /// The active overlay.
    pub const fn overlay(&self) -> Overlay {
        self.overlay
    }

    /// The quick-settings state.
    pub const fn quick(&self) -> &QuickState {
        &self.quick
    }

    /// Whether anything needs repainting.
    pub const fn needs_render(&self) -> bool {
        self.is_dirty
    }

    /// Forces the next frame to rebuild.
    pub fn invalidate(&mut self) {
        self.is_dirty = true;
    }

    /// Updates time and battery, returning whether the preview changed.
    pub fn update(&mut self, now: Timestamp, battery: Percent, is_charging: bool) -> bool {
        let changed = self.home.update(now, battery, is_charging);
        self.is_dirty |= changed;
        changed
    }

    /// Opens the app drawer.
    pub fn open_drawer(&mut self) {
        self.overlay = Overlay::Drawer;
        self.is_dirty = true;
    }

    /// Opens quick settings.
    pub fn open_quick_settings(&mut self) {
        self.overlay = Overlay::QuickSettings;
        self.is_dirty = true;
    }

    /// Closes the active overlay.
    pub fn close_overlay(&mut self) {
        if self.overlay != Overlay::None {
            self.overlay = Overlay::None;
            self.is_dirty = true;
        }
    }

    /// Handles a tap at screen coordinates.
    pub fn tap(&mut self, x: u16, y: u16, viewport: common::Region) -> bool {
        if self.dismiss_outside_overlay(y, viewport) {
            return true;
        }
        let Some(id) = self.hits.hit(x, y) else {
            return false;
        };
        if !self.overlay_owns(id) {
            return false;
        }
        self.activate(id, x)
    }

    fn dismiss_outside_overlay(&mut self, y: u16, viewport: common::Region) -> bool {
        match self.overlay {
            Overlay::Drawer if y < viewport.height.saturating_sub(drawer::HEIGHT) => {
                self.close_overlay();
                true
            }
            Overlay::QuickSettings if y > quick::HEIGHT => {
                self.close_overlay();
                true
            }
            _ => false,
        }
    }

    fn overlay_owns(&self, id: u16) -> bool {
        match self.overlay {
            Overlay::None => true,
            Overlay::Drawer => drawer::owns(id),
            Overlay::QuickSettings => quick::owns(id),
        }
    }

    fn activate(&mut self, id: u16, x: u16) -> bool {
        let handled = match id {
            drawer::SETTINGS_ID => {
                self.screen = Screen::Settings;
                self.overlay = Overlay::None;
                true
            }
            drawer::ALARMS_ID => {
                self.screen = Screen::Alarms;
                self.overlay = Overlay::None;
                true
            }
            drawer::GALLERY_ID => {
                self.screen = Screen::Gallery;
                self.overlay = Overlay::None;
                true
            }
            quick::WIFI_ID => {
                self.quick.wifi = !self.quick.wifi;
                true
            }
            quick::BLUETOOTH_ID => {
                self.quick.bluetooth = !self.quick.bluetooth;
                true
            }
            quick::DND_ID => {
                self.quick.dnd = !self.quick.dnd;
                true
            }
            quick::SYNC_ID => {
                self.quick.sync = !self.quick.sync;
                true
            }
            quick::BRIGHTNESS_ID => self.set_brightness(id, x),
            id if gallery::is_back(id) || placeholder::is_back(id) => {
                self.screen = Screen::Home;
                true
            }
            id if gallery::is_next(id) => {
                self.gallery_page = (self.gallery_page + 1) % PAGES;
                true
            }
            _ => false,
        };
        self.is_dirty |= handled;
        handled
    }

    fn set_brightness(&mut self, id: u16, x: u16) -> bool {
        let Some(target) = self.hits.targets().iter().find(|target| target.id == id) else {
            return false;
        };
        if target.rect.width == 0 {
            return false;
        }
        let offset = x
            .clamp(target.rect.x, target.rect.right())
            .saturating_sub(target.rect.x);
        let value = u32::from(offset) * 100 / u32::from(target.rect.width);
        self.quick.brightness = value.min(100) as u8;
        true
    }
}
