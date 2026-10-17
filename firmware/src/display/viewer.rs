//! Phase 4.2 input viewer.
//!
//! Draws touch points plus accelerometer and gyroscope samples so input can be
//! verified on the panel. This is bring-up output, not the Phase 5 UI.
// TODO(phase-5): replace with the app renderer and shared surface.

use common::{MotionSample, Region, TouchReport, MAX_TOUCH_POINTS};
use hal::Display as _;
use ui::{digits_size, draw_digits, draw_text, text_size, MonoBuffer};

use super::{BringUpError, Display};

const SCALE: u16 = 2;
const GAP: u16 = 6;
const INFO_DIGITS: u8 = 3;
const COORD_DIGITS: u8 = 4;
const AXIS_DIGITS: u8 = 4;
const AXIS_LINES: u16 = 6;

/// Values shown by the input viewer.
#[derive(Clone, Copy)]
pub struct Viewer<'a> {
    /// Latest touch snapshot.
    pub touch: &'a TouchReport,
    /// Latest accelerometer and gyroscope snapshot.
    pub motion: &'a MotionSample,
    /// IMU chip identity, when it could be read.
    pub imu_id: Option<u8>,
    /// IMU revision, when it could be read.
    pub revision: Option<u8>,
    /// Stable error code, when an input read failed.
    pub error: Option<u16>,
}

impl Display {
    /// Draws the combined touch and motion report centered on the panel.
    pub fn draw_input_viewer(&mut self, view: &Viewer<'_>) -> Result<(), BringUpError> {
        let title = "INPUT";
        let (title_width, line_height) = text_size(title, SCALE);

        let (error_label_width, _) = text_size("ERR ", SCALE);
        let (id_label_width, _) = text_size("IMU ", SCALE);
        let (id_digits_width, _) = digits_size(INFO_DIGITS, SCALE);
        let (slash_width, _) = text_size("/", SCALE);
        let info_width = (error_label_width + id_digits_width)
            .max(id_label_width + id_digits_width * 2 + slash_width);

        let (point_label_width, _) = text_size("P0 ", SCALE);
        let (coord_width, _) = digits_size(COORD_DIGITS, SCALE);
        let (comma_width, _) = text_size(",", SCALE);
        let point_width = point_label_width + coord_width * 2 + comma_width;

        let (axis_label_width, _) = text_size("AX ", SCALE);
        let (sign_width, _) = text_size("-", SCALE);
        let (axis_digits_width, _) = digits_size(AXIS_DIGITS, SCALE);
        let axis_width = axis_label_width + sign_width + axis_digits_width;

        let (empty_width, _) = text_size("NO TOUCH", SCALE);
        let width = title_width
            .max(info_width)
            .max(point_width)
            .max(axis_width)
            .max(empty_width);
        let lines = 2 + MAX_TOUCH_POINTS as u16 + AXIS_LINES;
        let height = line_height * lines + GAP * (lines - 1);

        let x = self.geometry.width.saturating_sub(width) / 2;
        let y = self.geometry.height.saturating_sub(height) / 2;
        let block = self.geometry.align_region(Region::new(x, y, width, height));
        let origin_x = x - block.x;
        let origin_y = y - block.y;

        let needs_frame = match self.viewer_frame.as_ref() {
            Some(frame) => frame.width() != block.width || frame.height() != block.height,
            None => true,
        };
        if needs_frame {
            self.viewer_frame = Some(MonoBuffer::new(block.width, block.height));
        }
        let Some(frame) = self.viewer_frame.as_mut() else {
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
        draw_info(frame, view, (origin_x, info_y), width);

        let body_y = info_y + line_height + GAP;
        draw_touch(
            frame,
            view.touch,
            (origin_x, body_y),
            width,
            point_width,
            empty_width,
        );
        draw_motion(
            frame,
            view.motion,
            (
                origin_x,
                body_y + MAX_TOUCH_POINTS as u16 * (line_height + GAP),
            ),
            width,
            axis_width,
        );

        let needed = block.rgb565_len();
        self.viewer_pixels.resize(needed, 0);
        frame.expand_region(
            Region::full(block.width, block.height),
            &mut self.viewer_pixels[..needed],
        );
        self.panel
            .write_region(block, &self.viewer_pixels[..needed])
            .map_err(BringUpError::Display)
    }
}

fn draw_info(frame: &mut MonoBuffer, view: &Viewer<'_>, origin: (u16, u16), width: u16) {
    if let Some(code) = view.error {
        let (label_width, _) = text_size("ERR ", SCALE);
        let (digits_width, _) = digits_size(INFO_DIGITS, SCALE);
        let line_width = label_width + digits_width;
        let value_x = origin.0 + (width - line_width) / 2;
        draw_text(frame, "ERR ", (value_x, origin.1), SCALE);
        draw_digits(
            frame,
            u32::from(code),
            INFO_DIGITS,
            (value_x + label_width, origin.1),
            SCALE,
        );
        return;
    }

    let (label_width, _) = text_size("IMU ", SCALE);
    let (digits_width, _) = digits_size(INFO_DIGITS, SCALE);
    let (slash_width, _) = text_size("/", SCALE);
    let line_width = label_width + digits_width * 2 + slash_width;
    let line_x = origin.0 + (width - line_width) / 2;
    draw_text(frame, "IMU ", (line_x, origin.1), SCALE);

    let mut cursor = line_x + label_width;
    match view.imu_id {
        Some(id) => {
            draw_digits(frame, u32::from(id), INFO_DIGITS, (cursor, origin.1), SCALE);
        }
        None => draw_text(frame, "---", (cursor, origin.1), SCALE),
    }
    cursor += digits_width;
    draw_text(frame, "/", (cursor, origin.1), SCALE);
    cursor += slash_width;
    match view.revision {
        Some(revision) => {
            draw_digits(
                frame,
                u32::from(revision),
                INFO_DIGITS,
                (cursor, origin.1),
                SCALE,
            );
        }
        None => draw_text(frame, "---", (cursor, origin.1), SCALE),
    }
}

fn draw_touch(
    frame: &mut MonoBuffer,
    touch: &TouchReport,
    origin: (u16, u16),
    width: u16,
    point_width: u16,
    empty_width: u16,
) {
    let (line_height, _) = text_size("P0 ", SCALE);
    let (point_label_width, _) = text_size("P0 ", SCALE);
    let (coord_width, _) = digits_size(COORD_DIGITS, SCALE);
    let (comma_width, _) = text_size(",", SCALE);

    if touch.is_empty() {
        draw_text(
            frame,
            "NO TOUCH",
            (origin.0 + (width - empty_width) / 2, origin.1),
            SCALE,
        );
        return;
    }

    for index in 0..MAX_TOUCH_POINTS {
        let Some(point) = touch.get(index) else {
            continue;
        };
        let line_y = origin.1 + index as u16 * (line_height + GAP);
        let line_x = origin.0 + (width - point_width) / 2;
        draw_text(frame, point_label(index), (line_x, line_y), SCALE);
        let cursor = line_x + point_label_width;
        draw_digits(
            frame,
            u32::from(point.x),
            COORD_DIGITS,
            (cursor, line_y),
            SCALE,
        );
        draw_text(frame, ",", (cursor + coord_width, line_y), SCALE);
        draw_digits(
            frame,
            u32::from(point.y),
            COORD_DIGITS,
            (cursor + coord_width + comma_width, line_y),
            SCALE,
        );
    }
}

fn draw_motion(
    frame: &mut MonoBuffer,
    motion: &MotionSample,
    origin: (u16, u16),
    width: u16,
    axis_width: u16,
) {
    let (line_height, _) = text_size("AX ", SCALE);
    let (label_width, _) = text_size("AX ", SCALE);

    let accel = &motion.acceleration;
    let angular = &motion.angular_rate;
    let rows = [
        ("AX ", i32::from(accel.x_mg)),
        ("AY ", i32::from(accel.y_mg)),
        ("AZ ", i32::from(accel.z_mg)),
        ("GX ", angular.x_mdps / 1000),
        ("GY ", angular.y_mdps / 1000),
        ("GZ ", angular.z_mdps / 1000),
    ];

    for (index, (label, value)) in rows.iter().enumerate() {
        let line_y = origin.1 + index as u16 * (line_height + GAP);
        let line_x = origin.0 + (width - axis_width) / 2;
        draw_text(frame, label, (line_x, line_y), SCALE);
        let cursor = line_x + label_width;
        draw_signed(frame, *value, AXIS_DIGITS, (cursor, line_y));
    }
}

fn draw_signed(frame: &mut MonoBuffer, value: i32, digits: u8, origin: (u16, u16)) {
    let (sign_width, _) = text_size("-", SCALE);
    let sign = if value < 0 { "-" } else { " " };
    draw_text(frame, sign, origin, SCALE);
    draw_digits(
        frame,
        value.unsigned_abs(),
        digits,
        (origin.0 + sign_width, origin.1),
        SCALE,
    );
}

fn point_label(index: usize) -> &'static str {
    match index {
        0 => "P0 ",
        1 => "P1 ",
        _ => "P? ",
    }
}
