//! Hardware preview screens: Home, overlays, and the settings screen.

mod calibrate;
mod chrome;
mod drawer;
mod engine;
mod gallery;
mod placeholder;
mod quick;
mod settings_actions;
mod settings_screen;

#[cfg(test)]
mod tests;

pub use engine::RenderError;
pub use gallery::PAGES;

use common::{Mode, Percent, Timestamp};
use settings::{SettingId, SettingValue, Settings};
use ui::{HitRegistry, Scene, TextCache};

use crate::home::HomeState;

/// Settings queued for the persistence owner.
const INTENT_CAPACITY: usize = 8;

/// The base screen currently shown by the preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Home.
    Home,
    /// Component gallery.
    Gallery,
    /// Persisted runtime settings.
    Settings,
    /// Alarms placeholder.
    Alarms,
    /// Panel shape calibration.
    Calibrate,
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
    settings: Settings,
    intents: [Option<(SettingId, SettingValue)>; INTENT_CAPACITY],
    intent_len: usize,
    settings_offset: u16,
    brightness_text: [u8; 4],
    brightness_len: usize,
    volume_text: [u8; 4],
    volume_len: usize,
    calibrate: calibrate::CalibrateState,
    gallery_page: u8,
    hits: HitRegistry,
    scene: Scene,
    cache: TextCache,
    is_dirty: bool,
}

impl Preview {
    /// Creates a preview booted to Home with a settings snapshot.
    pub fn new(
        mode: Mode,
        now: Timestamp,
        battery: Percent,
        is_charging: bool,
        settings: Settings,
    ) -> Self {
        let shape = ui::Theme::for_pixel_mode(pixel_mode(mode)).shape;
        let mut preview = Self {
            mode,
            screen: Screen::Home,
            overlay: Overlay::None,
            home: HomeState::new_with_format(
                now,
                battery,
                is_charging,
                settings.display.time_format,
            ),
            settings,
            intents: [None; INTENT_CAPACITY],
            intent_len: 0,
            settings_offset: 0,
            brightness_text: [0; 4],
            brightness_len: 0,
            volume_text: [0; 4],
            volume_len: 0,
            calibrate: calibrate::CalibrateState::new(shape),
            gallery_page: 0,
            hits: HitRegistry::new(),
            scene: Scene::new(),
            cache: TextCache::new(),
            is_dirty: true,
        };
        preview.refresh_text();
        preview
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

    /// Navigates back one level, closing an overlay first.
    pub fn back(&mut self) -> bool {
        if self.overlay != Overlay::None {
            self.close_overlay();
            return true;
        }
        if self.screen != Screen::Home {
            self.screen = Screen::Home;
            self.settings_offset = 0;
            self.is_dirty = true;
            return true;
        }
        false
    }

    /// Handles a tap at screen coordinates.
    pub fn tap(&mut self, x: u16, y: u16, _viewport: common::Region) -> bool {
        if self.dismiss_outside_overlay(y) {
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

    fn dismiss_outside_overlay(&mut self, y: u16) -> bool {
        match self.overlay {
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
        if self.screen == Screen::Calibrate && self.overlay == Overlay::None {
            return self.activate_calibrate(id);
        }
        if self.screen == Screen::Settings && self.overlay == Overlay::None {
            return self.activate_settings(id);
        }
        let handled = match id {
            drawer::SETTINGS_ID => {
                self.screen = Screen::Settings;
                self.overlay = Overlay::None;
                self.settings_offset = 0;
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
            drawer::CALIBRATE_ID => {
                self.screen = Screen::Calibrate;
                self.overlay = Overlay::None;
                true
            }
            quick::WIFI_ID => self.toggle(SettingId::ConnectivityWifi),
            quick::BLUETOOTH_ID => self.toggle(SettingId::ConnectivityBluetooth),
            quick::DND_ID => self.toggle(SettingId::SoundDnd),
            quick::SYNC_ID => self.toggle(SettingId::ConnectivitySync),
            quick::BRIGHTNESS_ID => self.set_brightness(id, x),
            id if gallery::is_next(id) => {
                self.gallery_page = (self.gallery_page + 1) % PAGES;
                true
            }
            _ => false,
        };
        self.is_dirty |= handled;
        handled
    }

    fn activate_calibrate(&mut self, id: u16) -> bool {
        let Some(action) = calibrate::action(id) else {
            return false;
        };
        self.calibrate.apply(action);
        self.is_dirty = true;
        true
    }
}

const fn pixel_mode(mode: Mode) -> common::PixelMode {
    match mode {
        Mode::Watch => common::PixelMode::Mono1,
        Mode::Desk => common::PixelMode::Gray8,
    }
}

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}
