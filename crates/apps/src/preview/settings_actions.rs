//! Settings interactions shared by the quick sheet and the settings screen.

use common::Percent;
use settings::{SettingId, SettingValue, Settings};
use ui::RedrawReason;

use super::{settings_screen, text, Overlay, Preview, Screen};
use crate::format;

impl Preview {
    /// The current settings snapshot.
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The formatted brightness for the active mode.
    pub fn brightness_text(&self) -> &str {
        text(&self.brightness_text, self.brightness_len)
    }

    /// The formatted output volume.
    pub fn volume_text(&self) -> &str {
        text(&self.volume_text, self.volume_len)
    }

    /// The current settings scroll offset.
    pub const fn settings_offset(&self) -> u16 {
        self.settings_offset
    }

    /// Removes the oldest setting queued for persistence.
    pub fn pop_setting(&mut self) -> Option<(SettingId, SettingValue)> {
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

    /// Scrolls the settings list by a signed pixel delta.
    pub fn scroll_settings(&mut self, delta: i16) -> bool {
        if self.screen != Screen::Settings || self.overlay != Overlay::None {
            return false;
        }
        let next = if delta >= 0 {
            self.settings_offset.saturating_add(delta as u16)
        } else {
            self.settings_offset.saturating_sub(delta.unsigned_abs())
        };
        let next = next.min(settings_screen::MAX_OFFSET);
        if next == self.settings_offset {
            return false;
        }
        self.settings_offset = next;
        self.is_dirty = true;
        true
    }

    pub(super) fn activate_settings(&mut self, id: u16) -> bool {
        let Some((setting, value)) = settings_screen::apply(id, &self.settings, self.mode) else {
            return false;
        };
        self.apply_setting(setting, value);
        true
    }

    pub(super) fn toggle(&mut self, id: SettingId) -> bool {
        let current = match id.read(&self.settings) {
            SettingValue::Bool(value) => value,
            _ => return false,
        };
        self.apply_setting(id, SettingValue::Bool(!current));
        true
    }

    fn apply_setting(&mut self, id: SettingId, value: SettingValue) {
        if id.apply(&mut self.settings, value).is_err() {
            return;
        }
        if let Some(slot) = self.intents.get_mut(self.intent_len) {
            *slot = Some((id, value));
            self.intent_len += 1;
        }
        if id == SettingId::DisplayTimeFormat {
            self.home.set_time_format(self.settings.display.time_format);
        }
        if id == SettingId::DisplayTheme {
            self.scene.request_full(RedrawReason::ThemeSwitch);
        }
        self.refresh_text();
        self.is_dirty = true;
    }

    pub(super) fn refresh_text(&mut self) {
        let brightness = self.settings.brightness(self.mode);
        let len = format::percent_text(brightness, &mut self.brightness_text).len();
        self.brightness_len = len;
        let len = format::percent_text(self.settings.sound.volume, &mut self.volume_text).len();
        self.volume_len = len;
    }

    pub(super) fn set_brightness(&mut self, id: u16, x: u16) -> bool {
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
        let setting = SettingId::brightness(self.mode);
        self.apply_setting(setting, SettingValue::Percent(Percent::new(value as u8)));
        true
    }
}
