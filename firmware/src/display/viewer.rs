//! Temporary trace viewer drawn on the panel.
//!
//! Shows storage state, the persisted boot counter, the latest log sequence,
//! and the schema version or an error code. This is bring-up output, not the
//! Phase 5 UI.

use alloc::vec;

use common::Region;
use hal::Display as _;
use ui::{digits_size, draw_digits, draw_text, text_size, MonoBuffer};

use super::{BringUpError, Display};

const SCALE: u16 = 2;
const GAP: u16 = 6;
const BOOT_DIGITS: u8 = 4;
const LOG_DIGITS: u8 = 4;
const VERSION_DIGITS: u8 = 2;
const ERROR_DIGITS: u8 = 3;

impl Display {
    /// Draws the storage viewer.
    pub fn draw_viewer(
        &mut self,
        status: &str,
        boot: u32,
        log: Option<u32>,
        version: u16,
        error: Option<u16>,
    ) -> Result<(), BringUpError> {
        let mut frame = MonoBuffer::new(self.geometry.width, self.geometry.height);

        let (status_width, line_height) = text_size(status, SCALE);
        let (boot_label_width, _) = text_size("BOOT ", SCALE);
        let (boot_width, _) = digits_size(BOOT_DIGITS, SCALE);
        let boot_line_width = boot_label_width + boot_width;

        let (log_label_width, log_line_width) = match log {
            Some(_) => {
                let (label_width, _) = text_size("LOG ", SCALE);
                let (value_width, _) = digits_size(LOG_DIGITS, SCALE);
                (label_width, label_width + value_width)
            }
            None => (0, text_size("LOG NONE", SCALE).0),
        };

        let (last_label, last_value, last_digits, last_label_width, last_width) = match error {
            Some(code) => {
                let (label_width, _) = text_size("ERR ", SCALE);
                let (value_width, _) = digits_size(ERROR_DIGITS, SCALE);
                (
                    "ERR ",
                    u32::from(code),
                    ERROR_DIGITS,
                    label_width,
                    label_width + value_width,
                )
            }
            None => {
                let (label_width, _) = text_size("VER ", SCALE);
                let (value_width, _) = digits_size(VERSION_DIGITS, SCALE);
                (
                    "VER ",
                    u32::from(version),
                    VERSION_DIGITS,
                    label_width,
                    label_width + value_width,
                )
            }
        };

        let width = status_width
            .max(boot_line_width)
            .max(log_line_width)
            .max(last_width);
        let height = line_height * 4 + GAP * 3;
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
        draw_text(&mut frame, "BOOT ", (boot_x, boot_y), SCALE);
        draw_digits(
            &mut frame,
            boot,
            BOOT_DIGITS,
            (boot_x + boot_label_width, boot_y),
            SCALE,
        );

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

        let last_y = log_y + line_height + GAP;
        let last_x = x + (width - last_width) / 2;
        draw_text(&mut frame, last_label, (last_x, last_y), SCALE);
        draw_digits(
            &mut frame,
            last_value,
            last_digits,
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
