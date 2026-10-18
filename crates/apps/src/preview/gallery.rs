//! A component gallery for tuning spacing, states, and typography.

use ui::{
    Align, BadgeProps, ButtonProps, CheckboxProps, IconButtonProps, IconId, InputProps, Insets,
    LabelProps, Length, ProgressProps, RadioProps, SliderProps, Status, StatusIndicatorProps,
    TabsProps, TextAlign, TextRole, TileProps, ToggleProps, Widget, WidgetKind,
};

use crate::view::{
    background, column, fill_node, fit_node, grid, leaf, node, row, spacer, BuildError, Builder,
};

/// Number of gallery pages.
pub const PAGES: u8 = 3;

const BACK_ID: u16 = 301;
const TITLE_ID: u16 = 302;
const NEXT_ID: u16 = 303;
const GRID_ID: u16 = 304;
const BG_ID: u16 = 300;
const WIDGET_BASE: u16 = 310;

/// Whether `id` is the back control.
pub const fn is_back(id: u16) -> bool {
    id == BACK_ID
}

/// Whether `id` is the next-page control.
pub const fn is_next(id: u16) -> bool {
    id == NEXT_ID
}

pub(super) fn build<'a>(page: u8, view: &mut Builder<'_, 'a>) -> Result<usize, BuildError> {
    let theme = view.theme;
    let back = Widget::new(
        BACK_ID,
        WidgetKind::IconButton(IconButtonProps {
            icon: IconId::CaretLeft,
            label: "Back",
        }),
    );
    let title = Widget::new(
        TITLE_ID,
        WidgetKind::Label(LabelProps {
            text: "Components",
            role: TextRole::Title,
            align: TextAlign::Start,
            icon: None,
        }),
    );
    let next = Widget::new(
        NEXT_ID,
        WidgetKind::Button(ButtonProps {
            label: "Next page",
            icon: None,
        }),
    );
    let cells = widgets_for(page);
    let content_width = view.viewport.width.saturating_sub(48);
    let cell = content_width.saturating_sub(theme.spacing.md) / 2;
    let back_node = fit_node(&back, theme, view.cache);
    let title_node = fill_node(&title, theme, view.cache, content_width);
    let next_node = fill_node(&next, theme, view.cache, content_width);
    let cell_nodes = [
        node(&cells[0], theme, view.cache, Length::Fill(1), cell),
        node(&cells[1], theme, view.cache, Length::Fill(1), cell),
        node(&cells[2], theme, view.cache, Length::Fill(1), cell),
        node(&cells[3], theme, view.cache, Length::Fill(1), cell),
    ];
    view.push(background(BG_ID))?;
    view.push(back)?;
    view.push(title)?;
    view.push(next)?;
    for widget in cells {
        view.push(widget)?;
    }

    let header_children = [back_node, title_node];
    let header = row(306, &header_children, theme.spacing.sm, Align::Center);
    let gallery = grid(GRID_ID, &cell_nodes, 2, theme.spacing.md, theme.spacing.md);
    let content_children = [
        header,
        spacer(307, Length::Fill(1), Length::Fixed(theme.spacing.md)),
        gallery,
        spacer(308, Length::Fill(1), Length::Fill(1)),
        next_node,
    ];
    let content = column(309, &content_children, theme.spacing.md, Align::Start)
        .with_padding(Insets::symmetric(24, 16));
    let root_children = [leaf(BG_ID, Length::Fill(1), Length::Fill(1)), content];
    let root = ui::Node::container(
        0,
        Length::Fill(1),
        Length::Fill(1),
        ui::Kind::Stack {
            align: Align::Start,
        },
        &root_children,
    );
    view.layout(&root)
}

fn widgets_for(page: u8) -> [Widget<'static>; 4] {
    match page % PAGES {
        0 => controls(),
        1 => inputs(),
        _ => data(),
    }
}

fn controls() -> [Widget<'static>; 4] {
    [
        Widget::new(
            WIDGET_BASE,
            WidgetKind::Button(ButtonProps {
                label: "Continue",
                icon: Some(IconId::Check),
            }),
        ),
        Widget::new(
            WIDGET_BASE + 1,
            WidgetKind::IconButton(IconButtonProps {
                icon: IconId::GearSix,
                label: "Settings",
            }),
        ),
        Widget::new(
            WIDGET_BASE + 2,
            WidgetKind::Tile(TileProps {
                icon: IconId::SquaresFour,
                label: "Gallery",
            }),
        ),
        Widget::new(
            WIDGET_BASE + 3,
            WidgetKind::StatusIndicator(StatusIndicatorProps {
                label: "Connected",
                status: Status::Ok,
            }),
        ),
    ]
}

fn inputs() -> [Widget<'static>; 4] {
    [
        Widget::new(
            WIDGET_BASE,
            WidgetKind::Toggle(ToggleProps {
                checked: true,
                label: "Wi-Fi",
            }),
        ),
        Widget::new(
            WIDGET_BASE + 1,
            WidgetKind::Checkbox(CheckboxProps {
                checked: true,
                label: "Sync",
            }),
        ),
        Widget::new(
            WIDGET_BASE + 2,
            WidgetKind::Radio(RadioProps {
                selected: true,
                label: "Option A",
            }),
        ),
        Widget::new(
            WIDGET_BASE + 3,
            WidgetKind::Slider(SliderProps {
                value: 60,
                label: "Volume",
            }),
        ),
    ]
}

fn data() -> [Widget<'static>; 4] {
    [
        Widget::new(
            WIDGET_BASE,
            WidgetKind::Input(InputProps {
                value: "hello",
                placeholder: "Name",
                masked: false,
                caret: None,
            }),
        ),
        Widget::new(
            WIDGET_BASE + 1,
            WidgetKind::Progress(ProgressProps {
                value: 60,
                label: "Sync",
                indeterminate: false,
            }),
        ),
        Widget::new(
            WIDGET_BASE + 2,
            WidgetKind::Badge(BadgeProps { label: "New" }),
        ),
        Widget::new(
            WIDGET_BASE + 3,
            WidgetKind::Tabs(TabsProps {
                labels: &["Home", "Apps", "Settings"],
                selected: 1,
            }),
        ),
    ]
}
