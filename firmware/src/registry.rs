//! Firmware-owned app registry.
//!
//! Registers Home, Settings, and Logs in both modes, plus the drawer and the
//! blank quick-settings overlay. The registry stays shared until an app needs
//! to be mode-exclusive.

use alloc::boxed::Box;

use apps::{
    AudioLab, Drawer, Home, LogsApp, QuickSettings, SettingsApp, AUDIO_LAB_APP_ID, LOGS_APP_ID,
    SETTINGS_APP_ID,
};
use common::{Mode, Percent, Timestamp};
use settings::{SettingId, SettingValue, Settings};
use shell::{App, AppId, AppRegistry, LayerKind};

/// The pinned Home route for both modes.
pub const HOME_ID: AppId = AppId::new(1);

/// A fixed registry with the watch app set.
pub struct Registry {
    home: Home,
    settings: SettingsApp,
    logs: LogsApp,
    audio_lab: Option<Box<AudioLab>>,
    drawer: Drawer,
    quick: QuickSettings,
}

impl Registry {
    /// Marks the alarm Home should show as ringing.
    pub fn set_alarm(&mut self, alarm: Option<Timestamp>) {
        self.home.set_alarm(alarm);
    }

    /// Creates the registry with a boot-time settings snapshot.
    pub fn new(
        mode: Mode,
        now: Timestamp,
        battery: Percent,
        is_charging: bool,
        settings: &Settings,
    ) -> Self {
        Self {
            home: Home::new_with_formats(
                mode,
                now,
                battery,
                is_charging,
                settings.display.time_format,
                settings.display.date_format,
            ),
            settings: SettingsApp::new(mode, *settings),
            logs: LogsApp::new(),
            audio_lab: None,
            drawer: Drawer::new(2, mode),
            quick: QuickSettings::new(),
        }
    }

    /// Applies authoritative settings to every app that caches them.
    pub fn apply_settings(&mut self, mode: Mode, settings: &Settings) {
        self.drawer.set_mode(mode);
        self.home.set_time_format(settings.display.time_format);
        self.home.set_date_format(settings.display.date_format);
        self.settings.set_mode(mode);
        self.settings.sync_settings(*settings);
    }

    /// Removes the oldest setting queued by the Settings app.
    pub fn pop_setting_intent(&mut self) -> Option<(SettingId, SettingValue)> {
        self.settings.pop_intent()
    }

    /// The log viewer, for loading a fresh snapshot.
    pub fn logs_mut(&mut self) -> &mut LogsApp {
        &mut self.logs
    }

    /// The desk audio tuner, when this session has one.
    pub fn audio_lab(&self) -> Option<&AudioLab> {
        self.audio_lab.as_deref()
    }

    /// The desk audio tuner, mutably.
    pub fn audio_lab_mut(&mut self) -> Option<&mut AudioLab> {
        self.audio_lab.as_deref_mut()
    }

    /// Allocates the desk tuner on the first desk entry.
    ///
    /// Watch boot never allocates the app. The firmware calls this after the
    /// rail and shell mode switch, or after desk boot has built the shell, so
    /// the app value is never constructed on the deep boot stack.
    pub fn ensure_audio_lab(&mut self) {
        if self.audio_lab.is_none() {
            self.audio_lab = Some(Box::new(AudioLab::new(Mode::Desk)));
        }
    }
}

impl AppRegistry for Registry {
    fn with_app<R>(&mut self, id: AppId, f: impl FnOnce(&mut dyn App) -> R) -> Option<R> {
        match id {
            HOME_ID => Some(f(&mut self.home)),
            SETTINGS_APP_ID => Some(f(&mut self.settings)),
            LOGS_APP_ID => Some(f(&mut self.logs)),
            AUDIO_LAB_APP_ID => self.audio_lab.as_deref_mut().map(|app| f(app)),
            _ => None,
        }
    }

    fn with_layer<R>(&mut self, kind: LayerKind, f: impl FnOnce(&mut dyn App) -> R) -> Option<R> {
        match kind {
            LayerKind::AppDrawer => Some(f(&mut self.drawer)),
            LayerKind::QuickSettings => Some(f(&mut self.quick)),
            _ => None,
        }
    }
}
