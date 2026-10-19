//! Unit tests for setting row value selection and formatting.

use super::rows::{next_sleep, seconds_text};

#[test]
fn sleep_timeout_advances_through_steps() {
    assert_eq!(next_sleep(30, 120), 60);
    assert_eq!(next_sleep(3_600, 120), 5);
    assert_eq!(next_sleep(7, 120), 10);
    assert_eq!(next_sleep(60, 60), 5);
}

#[test]
fn seconds_text_is_short_and_bounded() {
    let mut buffer = [0u8; 8];
    assert_eq!(seconds_text(5, &mut buffer), "5s");
    assert_eq!(seconds_text(3_600, &mut buffer), "3600s");
}
