//! Network status screen state and frame orchestration.

use common::{BleStatus, Region, WifiStatus};
use shell::{App, AppContext, AppError};
use ui::{
    Align, Damage, HitRegistry, LabelProps, Length, RedrawReason, Scene, SceneArena, SceneError,
    SceneNode, SettingsRowProps, SettingsTrailing, Surface, TextAlign, TextCache, TextRole, Theme,
    Widget, WidgetArena, WidgetKind,
};

use super::format;
use crate::view::{background, column, content_padding, crown, fill_node, BuildError, Builder};

const RENDER_FAILED_CODE: u16 = 1;
const BG_ID: u16 = 900;
const TITLE_ID: u16 = 902;
const CROWN_ID: u16 = 906;
const CONTENT_ID: u16 = 907;
const BODY_ID: u16 = 910;
const WIFI_ID: u16 = 920;
const BLE_ID: u16 = 921;

/// A request queued for the firmware-owned radio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    /// Start a WiFi connection attempt.
    WifiConnect,
    /// Stop WiFi.
    WifiStop,
    /// Start BLE advertising.
    BleStart,
    /// Stop BLE advertising.
    BleStop,
}

/// A Connectivity frame failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    /// A screen could not be built or laid out.
    Build,
    /// The scene arena overflowed.
    Scene(SceneError),
}

/// The desk-only network status app.
pub struct ConnectivityApp {
    wifi: WifiStatus,
    ble: BleStatus,
    wifi_text: format::RowText,
    ble_text: format::RowText,
    intent: Option<Intent>,
    hits: HitRegistry,
    cache: TextCache,
    scene: Scene,
}

impl ConnectivityApp {
    /// Creates the app in the disconnected state.
    pub fn new() -> Self {
        let wifi = WifiStatus::Off;
        let ble = BleStatus::Off;
        Self {
            wifi,
            ble,
            wifi_text: format::wifi(wifi),
            ble_text: format::ble(ble),
            intent: None,
            hits: HitRegistry::new(),
            cache: TextCache::new(),
            scene: Scene::new(),
        }
    }

    /// Replaces the firmware-supplied status snapshot.
    pub fn update(&mut self, wifi: WifiStatus, ble: BleStatus) -> bool {
        if self.wifi == wifi && self.ble == ble {
            return false;
        }
        self.wifi = wifi;
        self.ble = ble;
        self.wifi_text = format::wifi(wifi);
        self.ble_text = format::ble(ble);
        true
    }

    /// Removes the pending radio request.
    pub fn take_intent(&mut self) -> Option<Intent> {
        self.intent.take()
    }

    /// Builds, diffs, and paints the current frame.
    pub fn render_frame(
        &mut self,
        surface: &mut dyn Surface,
        theme: &Theme,
        viewport: Region,
    ) -> Result<Damage, RenderError> {
        let Self {
            wifi_text,
            ble_text,
            hits,
            cache,
            scene,
            ..
        } = self;
        let wifi_sub = wifi_text.text();
        let ble_sub = ble_text.text();
        let wifi_row = row(
            WIFI_ID,
            "WiFi",
            wifi_sub,
            ui::IconId::WifiHigh,
            wifi_text.action,
        );
        let ble_row = row(
            BLE_ID,
            "Bluetooth",
            ble_sub,
            ui::IconId::Bluetooth,
            ble_text.action,
        );
        let title = title();
        let mut widgets = WidgetArena::new();
        let mut entries = crate::view::entries();
        let count = build(
            &title,
            &wifi_row,
            &ble_row,
            theme,
            cache,
            &mut widgets,
            &mut entries,
            viewport,
        )
        .map_err(|_| RenderError::Build)?;
        ui::collect_hits(&widgets, &entries[..count], hits).map_err(|_| RenderError::Build)?;

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
        ui::paint_damage_shaped(
            surface,
            &damage,
            theme.tones.background,
            Some(theme.shape),
            |painter, _| ui::paint_widgets(painter, &widgets, &entries[..count], theme),
        );
        Ok(damage)
    }
}

impl Default for ConnectivityApp {
    fn default() -> Self {
        Self::new()
    }
}

impl App for ConnectivityApp {
    fn tap(&mut self, x: u16, y: u16, _context: &mut AppContext<'_>) -> Result<(), AppError> {
        match self.hits.hit(x, y) {
            Some(WIFI_ID) => {
                self.intent = if self.wifi.is_active() {
                    Some(Intent::WifiStop)
                } else if self.wifi == WifiStatus::Unconfigured {
                    None
                } else {
                    Some(Intent::WifiConnect)
                };
            }
            Some(BLE_ID) => {
                self.intent = if self.ble.is_active() {
                    Some(Intent::BleStop)
                } else {
                    Some(Intent::BleStart)
                };
            }
            _ => {}
        }
        Ok(())
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

fn build<'a>(
    title: &Widget<'a>,
    wifi: &Widget<'a>,
    ble: &Widget<'a>,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let title_node = fill_node(title, theme, cache, viewport.width);
    let row_nodes = [
        fill_node(wifi, theme, cache, viewport.width),
        fill_node(ble, theme, cache, viewport.width),
    ];

    let mut builder = Builder {
        theme,
        cache,
        widgets,
        entries,
        viewport,
    };
    builder.push(background(BG_ID))?;
    builder.push(*title)?;
    builder.push(*wifi)?;
    builder.push(*ble)?;

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let content = column(CONTENT_ID, &row_nodes, theme.spacing.xs, Align::Start)
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

fn row<'a>(
    id: u16,
    title: &'a str,
    subtitle: &'a str,
    icon: ui::IconId,
    action: &'a str,
) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::SettingsRow(SettingsRowProps {
            title,
            subtitle,
            icon: Some(icon),
            trailing: SettingsTrailing::Value(action),
            compact: false,
        }),
    )
}

fn title<'a>() -> Widget<'a> {
    Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Network",
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    )
}
