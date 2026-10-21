//! Host tests for the Audio Lab contracts.

use super::*;

#[test]
fn dbfs_uses_three_db_steps() {
    assert_eq!(
        Meter {
            peak: 32_768,
            ..Meter::SILENT
        }
        .peak_dbfs(),
        0
    );
    assert_eq!(
        Meter {
            peak: 16_384,
            ..Meter::SILENT
        }
        .peak_dbfs(),
        -6
    );
    assert_eq!(
        Meter {
            peak: 0,
            ..Meter::SILENT
        }
        .peak_dbfs(),
        -96
    );
}

#[test]
fn peak_percent_is_clamped() {
    assert_eq!(
        Meter {
            peak: 32_768,
            ..Meter::SILENT
        }
        .peak_percent(),
        100
    );
    assert_eq!(
        Meter {
            peak: 16_384,
            ..Meter::SILENT
        }
        .peak_percent(),
        50
    );
}

#[test]
fn rate_percent_reports_framing_errors() {
    let report = ClipReport {
        frames: 80_000,
        elapsed_ms: 5_000,
        ..ClipReport::default()
    };
    assert_eq!(report.rate_percent(), 100);
    let half = ClipReport {
        frames: 40_000,
        ..report
    };
    assert_eq!(half.rate_percent(), 50);
}
