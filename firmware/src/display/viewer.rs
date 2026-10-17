//! Phase 4.1 touch viewer.
//!
//! Draws the live FT3168 report and chip ID so coordinates can be verified on
//! the panel. This is bring-up output, not the Phase 5 UI.
// TODO(phase-5): replace with the app renderer and shared surface.

use common::{Region, TouchReport, MAX_TOUCH_POINTS};
use hal::Display as _;
use ui::{digits_size, draw_digits, draw_text, text_size, MonoBuffer};

use super::{BringUpError, Display};

const SCALE: u16 = 2;
const GAP: u16 = 6;
const INFO_DIGITS: u8 = 3;
const COORD_DIGITS: u8 = 4;

/// Values shown by the touch viewer.
#[derive(Clone, Copy)]
pub struct Viewer<'a> {
    /// Latest touch snapshot.
    pub report: &'a TouchReport,
    /// Chip identity, when it could be read.
    pub chip_id: Option<u8>,
    /// Stable error code, when touch bring-up or a read failed.
    pub error: Option<u16>,
}

impl Display {
    /// Draws the touch report centered on the panel.
    pub fn draw_touch_viewer(&mut self, view: &Viewer<'_>) -> Result<(), BringUpError> {
        let title = if view.error.is_some() {
            "TOUCH ERR"
        } else {
            "TOUCH"
        };
        let (title_width, line_height) = text_size(title, SCALE);
        let (max_title_width, _) = text_size("TOUCH ERR", SCALE);

        let (id_label_width, _) = text_size("ID ", SCALE);
        let (err_label_width, _) = text_size("ERR ", SCALE);
        let (info_digits_width, _) = digits_size(INFO_DIGITS, SCALE);
        let info_width = (id_label_width.max(err_label_width) + info_digits_width)
            .max(text_size("ID ---", SCALE).0);

        let (point_label_width, _) = text_size("P0 ", SCALE);
        let (coord_width, _) = digits_size(COORD_DIGITS, SCALE);
        let (comma_width, _) = text_size(",", SCALE);
        let point_width = point_label_width + coord_width * 2 + comma_width;

        let (empty_width, _) = text_size("NO TOUCH", SCALE);
        let width = max_title_width
            .max(info_width)
            .max(point_width)
            .max(empty_width);
        let lines = 2 + MAX_TOUCH_POINTS as u16;
        let height = line_height * lines + GAP * (lines - 1);

        let x = self.geometry.width.saturating_sub(width) / 2;
        let y = self.geometry.height.saturating_sub(height) / 2;
        let block = self.geometry.align_region(Region::new(x, y, width, height));
        let origin_x = x - block.x;
        let origin_y = y - block.y;

        let needs_frame = match self.touch_frame.as_ref() {
            Some(frame) => frame.width() != block.width || frame.height() != block.height,
            None => true,
        };
        if needs_frame {
            self.touch_frame = Some(MonoBuffer::new(block.width, block.height));
        }
        let Some(frame) = self.touch_frame.as_mut() else {
            return Ok(());
        };
        frame.clear();

        draw_text(
            frame,
            title,
            (origin_x + (width - title_width) / 2, origin_y),
            SCALE,
        );

        let info_y = origin_y + line_height + GAP;
        let info_value = view
            .error
            .map(u32::from)
            .or_else(|| view.chip_id.map(u32::from));
        match info_value {
            Some(value) => {
                let (label, label_width) = if view.error.is_some() {
                    ("ERR ", err_label_width)
                } else {
                    ("ID ", id_label_width)
                };
                let line_width = label_width + info_digits_width;
                let info_x = origin_x + (width - line_width) / 2;
                draw_text(frame, label, (info_x, info_y), SCALE);
                draw_digits(
                    frame,
                    value,
                    INFO_DIGITS,
                    (info_x + label_width, info_y),
                    SCALE,
                );
            }
            None => {
                let (label_width, _) = text_size("ID ---", SCALE);
                draw_text(
                    frame,
                    "ID ---",
                    (origin_x + (width - label_width) / 2, info_y),
                    SCALE,
                );
            }
        }

        let body_y = info_y + line_height + GAP;
        if view.report.is_empty() {
            draw_text(
                frame,
                "NO TOUCH",
                (origin_x + (width - empty_width) / 2, body_y),
                SCALE,
            );
        } else {
            for index in 0..MAX_TOUCH_POINTS {
                let Some(point) = view.report.get(index) else {
                    continue;
                };
                let line_y = body_y + index as u16 * (line_height + GAP);
                let line_x = origin_x + (width - point_width) / 2;
                draw_text(frame, point_label(index), (line_x, line_y), SCALE);
                let x_cursor = line_x + point_label_width;
                draw_digits(
                    frame,
                    u32::from(point.x),
                    COORD_DIGITS,
                    (x_cursor, line_y),
                    SCALE,
                );
                draw_text(frame, ",", (x_cursor + coord_width, line_y), SCALE);
                draw_digits(
                    frame,
                    u32::from(point.y),
                    COORD_DIGITS,
                    (x_cursor + coord_width + comma_width, line_y),
                    SCALE,
                );
            }
        }

        let needed = block.rgb565_len();
        self.touch_pixels.resize(needed, 0);
        frame.expand_region(
            Region::full(block.width, block.height),
            &mut self.touch_pixels[..needed],
        );
        self.panel
            .write_region(block, &self.touch_pixels[..needed])
            .map_err(BringUpError::Display)
    }
}

fn point_label(index: usize) -> &'static str {
    match index {
        0 => "P0 ",
        1 => "P1 ",
        _ => "P? ",
    }
}
