//! Per-component watch pixel budget tests.

use crate::widgets::WidgetState;

use super::{render_watch, SAMPLE_KINDS};

const WIDTH: u16 = 280;
const HEIGHT: u16 = 160;
const BUDGET_PERCENT: u32 = 30;

#[test]
fn every_component_stays_inside_the_watch_budget() {
    let area = u32::from(WIDTH) * u32::from(HEIGHT);
    let budget = area * BUDGET_PERCENT / 100;
    for (index, kind) in SAMPLE_KINDS.into_iter().enumerate() {
        let (_, lit) = render_watch(kind, WidgetState::NORMAL, WIDTH, HEIGHT);
        assert!(
            lit <= budget,
            "component {index} lit {lit} pixels, budget {budget}"
        );
    }
}

#[test]
fn a_focus_ring_stays_inside_the_budget() {
    let area = u32::from(WIDTH) * u32::from(HEIGHT);
    let budget = area * BUDGET_PERCENT / 100;
    let (_, lit) = render_watch(SAMPLE_KINDS[0], WidgetState::FOCUSED, WIDTH, HEIGHT);
    assert!(lit <= budget);
}
