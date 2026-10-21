//! Frame building for the Audio Lab pages.

use common::Region;
use ui::{
    Align, ButtonProps, IconId, LabelProps, Length, Node, ProgressProps, SettingsRowProps,
    SettingsTrailing, TabsProps, TextAlign, TextCache, TextRole, Theme, Widget, WidgetArena,
    WidgetKind,
};

use super::app::{AudioLab, Page};
use crate::view::{
    background, column, content_padding, crown, fill_node, grid, node, row, spacer, BuildError,
    Builder,
};

const BG_ID: u16 = 1_200;
const TITLE_ID: u16 = 1_201;
const CROWN_ID: u16 = 1_202;
const CONTENT_ID: u16 = 1_203;
const BODY_ID: u16 = 1_204;
const TABS_ID: u16 = 1_205;
const STATUS_ID: u16 = 1_206;
const ERROR_ID: u16 = 1_207;
const DETAIL_ID: u16 = 1_208;
const METERS_ID: u16 = 1_209;
const SETTINGS_ID: u16 = 1_212;
const MIC1_BAR_ID: u16 = 1_210;
const MIC2_BAR_ID: u16 = 1_211;
const ACTIONS_ID: u16 = 1_220;
const DURATION_ID: u16 = 1_230;
const GAIN_ID: u16 = 1_231;
const CHANNEL_ID: u16 = 1_232;
const VOLUME_ID: u16 = 1_233;
const MUTE_ID: u16 = 1_234;
const RECORD_ID: u16 = 1_240;
const PLAY_ID: u16 = 1_241;
const TONE_TEST_ID: u16 = 1_242;
const TONE_1K_ID: u16 = 1_243;
const TONE_440_ID: u16 = 1_244;

/// Tab strip hit target.
pub(super) const TABS_HIT_ID: u16 = TABS_ID;
/// Record or stop button.
pub(super) const RECORD_HIT_ID: u16 = RECORD_ID;
/// Play the last clip.
pub(super) const PLAY_HIT_ID: u16 = PLAY_ID;
/// Record with a 1 kHz test tone.
pub(super) const TONE_TEST_HIT_ID: u16 = TONE_TEST_ID;
/// Record length row.
pub(super) const DURATION_HIT_ID: u16 = DURATION_ID;
/// Microphone gain row.
pub(super) const GAIN_HIT_ID: u16 = GAIN_ID;
/// Playback microphone row.
pub(super) const CHANNEL_HIT_ID: u16 = CHANNEL_ID;
/// Volume row.
pub(super) const VOLUME_HIT_ID: u16 = VOLUME_ID;
/// Mute row.
pub(super) const MUTE_HIT_ID: u16 = MUTE_ID;
/// 1 kHz tone button.
pub(super) const TONE_1K_HIT_ID: u16 = TONE_1K_ID;
/// 440 Hz tone button.
pub(super) const TONE_440_HIT_ID: u16 = TONE_440_ID;

struct PageView<'w, 'a, 'n> {
    widgets: &'w [Widget<'a>],
    nodes: &'n [Node<'n>],
}

/// Builds the active page.
pub(super) fn build<'a, 's: 'a>(
    state: &'s AudioLab,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    match state.page {
        Page::Loopback => build_loopback(state, theme, cache, widgets, entries, viewport),
        Page::Mic => build_mic(state, theme, cache, widgets, entries, viewport),
        Page::Speaker => build_speaker(state, theme, cache, widgets, entries, viewport),
    }
}

fn build_loopback<'a, 's: 'a>(
    state: &'s AudioLab,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let mic1 = progress(
        MIC1_BAR_ID,
        text(&state.mic1_text, state.mic1_len),
        state.mic1,
    );
    let mic2 = progress(
        MIC2_BAR_ID,
        text(&state.mic2_text, state.mic2_len),
        state.mic2,
    );
    let duration = setting(
        DURATION_ID,
        "Record",
        "Tap to cycle",
        IconId::Clock,
        text(&state.duration_text, state.duration_len),
    );
    let record = button(
        RECORD_ID,
        if state.is_recording { "Stop" } else { "Rec" },
        IconId::Microphone,
    );
    let play = button(PLAY_ID, "Play", IconId::Play);
    let tone = button(TONE_TEST_ID, "Tone", IconId::SpeakerHigh);
    let meter_nodes = [
        fill_node(&mic1, theme, cache, 180),
        fill_node(&mic2, theme, cache, 180),
    ];
    let meters = grid(METERS_ID, &meter_nodes, 2, theme.spacing.sm, 0);
    let action_nodes = [
        node(&record, theme, cache, Length::Fill(1), theme.min_touch_px),
        node(&play, theme, cache, Length::Fill(1), theme.min_touch_px),
        node(&tone, theme, cache, Length::Fill(1), theme.min_touch_px),
    ];
    let actions = row(ACTIONS_ID, &action_nodes, theme.spacing.sm, Align::Center);
    let nodes = [
        meters,
        fill_node(&duration, theme, cache, content_width(theme, viewport)),
        actions,
    ];
    let page_widgets = [mic1, mic2, duration, record, play, tone];
    let page = PageView {
        widgets: &page_widgets,
        nodes: &nodes,
    };
    finish(state, theme, cache, widgets, entries, viewport, &page)
}

fn build_mic<'a, 's: 'a>(
    state: &'s AudioLab,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let gain = setting(
        GAIN_ID,
        "Gain",
        "3 dB",
        IconId::Microphone,
        text(&state.gain_text, state.gain_len),
    );
    let channel = setting(
        CHANNEL_ID,
        "Mic",
        "Source",
        IconId::SpeakerHigh,
        state.channel.label(),
    );
    let mic1 = progress(
        MIC1_BAR_ID,
        text(&state.mic1_text, state.mic1_len),
        state.mic1,
    );
    let mic2 = progress(
        MIC2_BAR_ID,
        text(&state.mic2_text, state.mic2_len),
        state.mic2,
    );
    let record = button(
        RECORD_ID,
        if state.is_recording { "Stop" } else { "Rec" },
        IconId::Microphone,
    );
    let play = button(PLAY_ID, "Play", IconId::Play);
    let meter_nodes = [
        fill_node(&mic1, theme, cache, 180),
        fill_node(&mic2, theme, cache, 180),
    ];
    let meters = grid(METERS_ID, &meter_nodes, 2, theme.spacing.sm, 0);
    let action_nodes = [
        node(&record, theme, cache, Length::Fill(1), theme.min_touch_px),
        node(&play, theme, cache, Length::Fill(1), theme.min_touch_px),
    ];
    let actions = row(ACTIONS_ID, &action_nodes, theme.spacing.sm, Align::Center);
    let setting_nodes = [
        fill_node(&gain, theme, cache, 180),
        fill_node(&channel, theme, cache, 180),
    ];
    let settings = grid(SETTINGS_ID, &setting_nodes, 2, theme.spacing.sm, 0);
    let nodes = [settings, meters, actions];
    let page_widgets = [gain, channel, mic1, mic2, record, play];
    let page = PageView {
        widgets: &page_widgets,
        nodes: &nodes,
    };
    finish(state, theme, cache, widgets, entries, viewport, &page)
}

fn build_speaker<'a, 's: 'a>(
    state: &'s AudioLab,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
) -> Result<usize, BuildError> {
    let volume = setting(
        VOLUME_ID,
        "Volume",
        "Tap to cycle",
        IconId::SpeakerHigh,
        text(&state.volume_text, state.volume_len),
    );
    let mute = Widget::new(
        MUTE_ID,
        WidgetKind::SettingsRow(SettingsRowProps {
            title: "Mute",
            subtitle: "Tap to toggle",
            icon: Some(IconId::SpeakerHigh),
            trailing: SettingsTrailing::Toggle(state.muted),
            compact: true,
        }),
    );
    let low = button(TONE_440_ID, "440 Hz", IconId::SpeakerHigh);
    let high = button(TONE_1K_ID, "1 kHz", IconId::SpeakerHigh);
    let play = button(PLAY_ID, "Play", IconId::Play);
    let action_nodes = [
        node(&low, theme, cache, Length::Fill(1), theme.min_touch_px),
        node(&high, theme, cache, Length::Fill(1), theme.min_touch_px),
        node(&play, theme, cache, Length::Fill(1), theme.min_touch_px),
    ];
    let actions = row(ACTIONS_ID, &action_nodes, theme.spacing.sm, Align::Center);
    let width = content_width(theme, viewport);
    let nodes = [
        fill_node(&volume, theme, cache, width),
        fill_node(&mute, theme, cache, width),
        actions,
    ];
    let page_widgets = [volume, mute, low, high, play];
    let page = PageView {
        widgets: &page_widgets,
        nodes: &nodes,
    };
    finish(state, theme, cache, widgets, entries, viewport, &page)
}

fn finish<'w, 'a, 's: 'a, 'n>(
    state: &'s AudioLab,
    theme: &Theme,
    cache: &mut TextCache,
    widgets: &mut WidgetArena<'a>,
    entries: &mut [ui::LayoutEntry],
    viewport: Region,
    page: &PageView<'w, 'a, 'n>,
) -> Result<usize, BuildError> {
    let title = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Audio Lab",
            role: TextRole::Title,
            align: TextAlign::Center,
            icon: None,
        }),
    );
    let tabs = Widget::new(
        TABS_ID,
        WidgetKind::Tabs(TabsProps {
            labels: &["Loop", "Mic", "Speaker"],
            selected: state.page.index(),
        }),
    );
    let status = label(
        STATUS_ID,
        text(&state.status_text, state.status_len),
        TextRole::Caption,
        TextAlign::Start,
    );
    let detail = label(
        DETAIL_ID,
        text(&state.detail_text, state.detail_len),
        TextRole::Caption,
        TextAlign::Start,
    );
    let error = (state.error_len > 0).then(|| {
        label(
            ERROR_ID,
            text(&state.error_text, state.error_len),
            TextRole::Caption,
            TextAlign::Start,
        )
    });

    let width = content_width(theme, viewport);
    let title_node = fill_node(&title, theme, cache, width);
    let tabs_node = fill_node(&tabs, theme, cache, width);
    let status_node = fill_node(&status, theme, cache, width);
    let detail_node = fill_node(&detail, theme, cache, width);
    let error_node = error.as_ref().map(|w| fill_node(w, theme, cache, width));

    let mut body_children: [Node<'n>; 12] = [spacer(0, Length::Fill(1), Length::Fixed(0)); 12];
    let mut count = 0;
    body_children[count] = tabs_node;
    count += 1;
    body_children[count] = status_node;
    count += 1;
    if let Some(error_node) = error_node {
        body_children[count] = error_node;
        count += 1;
    }
    body_children[count] = detail_node;
    count += 1;
    for node in page.nodes {
        if count >= body_children.len() {
            break;
        }
        body_children[count] = *node;
        count += 1;
    }

    let mut builder = Builder {
        theme,
        cache,
        widgets,
        entries,
        viewport,
    };
    builder.push(background(BG_ID))?;
    builder.push(title)?;
    builder.push(tabs)?;
    builder.push(status)?;
    builder.push(detail)?;
    if let Some(error) = error {
        builder.push(error)?;
    }
    for widget in page.widgets {
        builder.push(*widget)?;
    }

    let crown_children = [title_node];
    let crown_node = crown(CROWN_ID, &crown_children, theme);
    let content = column(
        CONTENT_ID,
        &body_children[..count],
        theme.spacing.xs,
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

fn progress<'a>(id: u16, label: &'a str, meter: voice::lab::Meter) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::Progress(ProgressProps {
            value: meter.peak_percent(),
            label,
            indeterminate: false,
        }),
    )
}

fn setting<'a>(
    id: u16,
    title: &'a str,
    subtitle: &'a str,
    icon: IconId,
    value: &'a str,
) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::SettingsRow(SettingsRowProps {
            title,
            subtitle,
            icon: Some(icon),
            trailing: SettingsTrailing::Value(value),
            compact: true,
        }),
    )
}

fn button<'a>(id: u16, label: &'a str, icon: IconId) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::Button(ButtonProps {
            label,
            icon: Some(icon),
        }),
    )
}

fn label<'a>(id: u16, value: &'a str, role: TextRole, align: TextAlign) -> Widget<'a> {
    Widget::new(
        id,
        WidgetKind::Label(LabelProps {
            text: value,
            role,
            align,
            icon: None,
        }),
    )
}

fn content_width(theme: &Theme, viewport: Region) -> u16 {
    viewport
        .width
        .saturating_sub(theme.shape.content_inset.saturating_mul(2))
}

fn text(bytes: &[u8], len: usize) -> &str {
    core::str::from_utf8(&bytes[..len.min(bytes.len())]).unwrap_or("")
}
