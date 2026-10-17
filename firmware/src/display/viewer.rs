//! Temporary trace viewer drawn on the panel.
//!
//! Shows storage state, the persisted boot counter, the latest log sequence,
//! the boot battery sample, and either buffered/dropped counts or an error
//! code. This is bring-up output, not the Phase 5 UI.

use alloc::vec;

use common::Region;
use hal::Display as _;
use ui::{digits_size, draw_digits, draw_text, text_size, MonoBuffer};

use super::{BringUpError, Display};

const SCALE: u16 = 2;
const GAP: u16 = 6;
const BOOT_DIGITS: u8 = 4;
const LOG_DIGITS: u8 = 4;
const BATTERY_DIGITS: u8 = 4;
const COUNT_DIGITS: u8 = 3;
const ERROR_DIGITS: u8 = 3;

/// Values shown by the trace viewer.
#[derive(Clone, Copy)]
pub struct Viewer<'a> {
    /// Key-value store status.
    pub status: &'a str,
    /// Persisted trace boot counter.
    pub boot: u32,
    /// Latest persisted log sequence.
    pub log: Option<u32>,
    /// Battery voltage read this boot.
    pub battery_mv: u16,
    /// Records waiting in retained memory.
    pub buffered: u16,
    /// Records dropped by ring overflow.
    pub dropped: u16,
    /// Storage error code, when the boot failed.
    pub error: Option<u16>,
}

impl Display {
    /// Draws the storage and logging viewer.
    pub fn draw_viewer(&mut self, view: &Viewer<'_>) -> Result<(), BringUpError> {
        let Viewer {
            status,
            boot,
            log,
            battery_mv,
            buffered,
            dropped,
            error,
        } = *view;
        let mut frame = MonoBuffer::new(self.geometry.width, self.geometry.height);

        let (status_width, line_height) = text_size(status, SCALE);
        let (boot_line_width, boot_label_width) = if boot == 0 {
            (text_size("BOOT ----", SCALE).0, 0)
        } else {
            let (label_width, _) = text_size("BOOT ", SCALE);
            let (value_width, _) = digits_size(BOOT_DIGITS, SCALE);
            (label_width + value_width, label_width)
        };

        let (log_label_width, log_line_width) = match log {
            Some(_) => {
                let (label_width, _) = text_size("LOG ", SCALE);
                let (value_width, _) = digits_size(LOG_DIGITS, SCALE);
                (label_width, label_width + value_width)
            }
            None => (0, text_size("LOG NONE", SCALE).0),
        };

        let (battery_label_width, _) = text_size("BATT ", SCALE);
        let (battery_width, _) = digits_size(BATTERY_DIGITS, SCALE);
        let battery_line_width = battery_label_width + battery_width;

        let (last_label, last_value, last_label_width, last_width) = match error {
            Some(code) => count_line("ERR ", u32::from(code), ERROR_DIGITS),
            None if dropped > 0 => count_line("DRP ", u32::from(dropped), COUNT_DIGITS),
            None => count_line("BUF ", u32::from(buffered), COUNT_DIGITS),
        };

        let width = status_width
            .max(boot_line_width)
            .max(log_line_width)
            .max(battery_line_width)
            .max(last_width);
        let height = line_height * 5 + GAP * 4;
        let x = self.geometry.width.saturating_sub(width) / 2;
        let y = self.geometry.height.saturating_sub(height) / 2;

        draw_text(
            &mut frame,
            status,
            (x + (width - status_width) / 2, y),
            SCALE,
        );

        let boot_y = y + line_height + GAP;
        let boot_x = x + (width - boot_line_width) / 2;
        if boot == 0 {
            draw_text(&mut frame, "BOOT ----", (boot_x, boot_y), SCALE);
        } else {
            draw_text(&mut frame, "BOOT ", (boot_x, boot_y), SCALE);
            draw_digits(
                &mut frame,
                boot,
                BOOT_DIGITS,
                (boot_x + boot_label_width, boot_y),
                SCALE,
            );
        }

        let log_y = boot_y + line_height + GAP;
        let log_x = x + (width - log_line_width) / 2;
        match log {
            Some(sequence) => {
                draw_text(&mut frame, "LOG ", (log_x, log_y), SCALE);
                draw_digits(
                    &mut frame,
                    sequence,
                    LOG_DIGITS,
                    (log_x + log_label_width, log_y),
                    SCALE,
                );
            }
            None => draw_text(&mut frame, "LOG NONE", (log_x, log_y), SCALE),
        }

        let battery_y = log_y + line_height + GAP;
        let battery_x = x + (width - battery_line_width) / 2;
        draw_text(&mut frame, "BATT ", (battery_x, battery_y), SCALE);
        draw_digits(
            &mut frame,
            u32::from(battery_mv),
            BATTERY_DIGITS,
            (battery_x + battery_label_width, battery_y),
            SCALE,
        );

        let last_y = battery_y + line_height + GAP;
        let last_x = x + (width - last_width) / 2;
        draw_text(&mut frame, last_label, (last_x, last_y), SCALE);
        draw_digits(
            &mut frame,
            last_value,
            COUNT_DIGITS,
            (last_x + last_label_width, last_y),
            SCALE,
        );

        let block = self.geometry.align_region(Region::new(x, y, width, height));
        let mut expanded = vec![0u8; block.rgb565_len()];
        frame.expand_region(block, &mut expanded);

        self.panel
            .write_region(block, &expanded)
            .map_err(BringUpError::Display)
    }
}

fn count_line(label: &'static str, value: u32, digits: u8) -> (&'static str, u32, u16, u16) {
    let (label_width, _) = text_size(label, SCALE);
    let (value_width, _) = digits_size(digits, SCALE);
    (label, value, label_width, label_width + value_width)
}
