//! Shape-aware chrome accents drawn above the widgets.

use common::Region;
use ui::{Painter, Theme};

use super::{Overlay, Screen};

const HOME_ARC_INSET_PX: u16 = 48;

pub(super) fn paint(
    painter: &mut Painter<'_>,
    theme: &Theme,
    screen: Screen,
    overlay: Overlay,
    viewport: Region,
) {
    if overlay != Overlay::None || screen == Screen::Calibrate {
        return;
    }
    match screen {
        Screen::Home => paint_home_corners(painter, theme, viewport),
        _ => paint_back_hint(painter, theme, viewport),
    }
}

fn paint_home_corners(painter: &mut Painter<'_>, theme: &Theme, viewport: Region) {
    let shape = theme.shape;
    let radius = shape.corner_radius.saturating_sub(HOME_ARC_INSET_PX);
    if radius == 0 {
        return;
    }
    let right = viewport.width.saturating_sub(shape.corner_radius);
    painter.arc_quadrant(
        (shape.corner_radius, shape.corner_radius),
        radius,
        false,
        false,
        theme.tones.border,
    );
    painter.arc_quadrant(
        (right, shape.corner_radius),
        radius,
        true,
        false,
        theme.tones.border,
    );
}

fn paint_back_hint(painter: &mut Painter<'_>, theme: &Theme, viewport: Region) {
    let middle = viewport.height / 2;
    let x = theme.shape.content_inset / 2;
    painter.line(x + 6, middle - 7, x, middle, theme.tones.text_muted);
    painter.line(x, middle, x + 6, middle + 7, theme.tones.text_muted);
}
