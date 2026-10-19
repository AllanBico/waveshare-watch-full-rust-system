//! Host tests for the power manager against a fake PMIC.

use common::{Millivolts, Mode, WakeReason};
use hal::{Power, Rail};

use super::plan::ENABLE_ORDER;
use super::{BudgetStatus, PowerError, PowerManager, PowerState};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FakeError;

struct FakePower {
    enabled: [bool; 11],
    calls: Vec<(Rail, bool)>,
    fail: bool,
    lie: bool,
    battery: u16,
    vbus: bool,
}

impl FakePower {
    fn new() -> Self {
        Self {
            enabled: [false; 11],
            calls: Vec::new(),
            fail: false,
            lie: false,
            battery: 3800,
            vbus: false,
        }
    }

    fn clear_calls(&mut self) {
        self.calls.clear();
    }
}

impl Power for FakePower {
    type Error = FakeError;

    fn set_rail_enabled(&mut self, rail: Rail, enabled: bool) -> Result<(), Self::Error> {
        if self.fail {
            return Err(FakeError);
        }
        self.enabled[rail.index() as usize] = enabled;
        self.calls.push((rail, enabled));
        Ok(())
    }

    fn is_rail_enabled(&mut self, rail: Rail) -> Result<bool, Self::Error> {
        let real = self.enabled[rail.index() as usize];
        Ok(if self.lie { !real } else { real })
    }

    fn set_rail_voltage(
        &mut self,
        _rail: Rail,
        _millivolts: Millivolts,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    fn read_battery_millivolts(&mut self) -> Result<Millivolts, Self::Error> {
        Ok(Millivolts::new(self.battery))
    }

    fn is_vbus_present(&mut self) -> Result<bool, Self::Error> {
        Ok(self.vbus)
    }

    fn power_off(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn manager() -> PowerManager<FakePower> {
    PowerManager::new(FakePower::new(), Mode::Watch)
}

#[test]
fn new_defaults_to_active_watch() {
    let pm = manager();
    assert_eq!(pm.state(), PowerState::ActiveWatch);
    assert_eq!(pm.mode(), Mode::Watch);
    assert!(pm.last_wake().is_none());
}

#[test]
fn sleep_cuts_peripheral_rail_and_keeps_system() {
    let mut pm = manager();
    pm.enter_sleep().unwrap();
    assert_eq!(pm.state(), PowerState::DeepSleep);
    assert!(pm.pmic_mut().enabled[Rail::Dcdc1.index() as usize]);
    assert!(!pm.pmic_mut().enabled[Rail::Aldo3.index() as usize]);
    assert!(!pm.pmic_mut().enabled[Rail::Aldo2.index() as usize]);
}

#[test]
fn awake_then_sleep_sequences_offs_after_ons() {
    let mut pm = manager();
    pm.request_awake(WakeReason::Motion, 5_000, 0).unwrap();
    let on_rails: Vec<Rail> = pm.pmic_mut().calls.iter().map(|c| c.0).collect();
    assert_eq!(
        on_rails,
        vec![
            Rail::Dcdc1,
            Rail::Dcdc3,
            Rail::Dcdc4,
            Rail::Aldo1,
            Rail::Aldo2,
            Rail::Aldo3,
            Rail::Aldo4,
            Rail::Bldo1,
        ]
    );
    assert!(pm.pmic_mut().calls.iter().all(|c| c.1));

    pm.pmic_mut().clear_calls();
    pm.enter_sleep().unwrap();
    let off_calls: Vec<(Rail, bool)> = pm.pmic_mut().calls.clone();
    assert_eq!(
        off_calls,
        vec![
            (Rail::Bldo1, false),
            (Rail::Aldo3, false),
            (Rail::Aldo2, false),
            (Rail::Dcdc4, false),
            (Rail::Dcdc3, false),
        ]
    );
}

#[test]
fn budget_expires() {
    let mut pm = manager();
    pm.request_awake(WakeReason::Timer, 100, 1_000).unwrap();
    assert_eq!(pm.check_budget(1_099), BudgetStatus::Awake);
    assert_eq!(pm.check_budget(1_100), BudgetStatus::Expired);
    pm.enter_sleep().unwrap();
    assert_eq!(pm.check_budget(999_999), BudgetStatus::Awake);
}

#[test]
fn a_hundred_wake_cycles_expire_into_sleep() {
    let mut pm = manager();
    for cycle in 0..100u64 {
        let now = cycle * 1_000;
        pm.request_awake(WakeReason::Timer, 100, now).unwrap();
        assert_eq!(pm.check_budget(now + 99), BudgetStatus::Awake);
        assert_eq!(pm.check_budget(now + 100), BudgetStatus::Expired);
        pm.enter_sleep().unwrap();
        assert_eq!(pm.check_budget(now + 1_000_000), BudgetStatus::Awake);
    }
}

#[test]
fn audit_detects_lie() {
    let mut pm = PowerManager::new(FakePower::new(), Mode::Watch);
    pm.pmic_mut().lie = true;
    assert_eq!(pm.enter_sleep().unwrap_err(), PowerError::AuditFailed);
}

#[test]
fn pmic_error_propagates() {
    let mut pm = PowerManager::new(FakePower::new(), Mode::Watch);
    pm.pmic_mut().fail = true;
    assert_eq!(pm.enter_sleep().unwrap_err(), PowerError::Pmic(FakeError));
}

#[test]
fn enable_order_covers_every_rail() {
    assert_eq!(ENABLE_ORDER.len(), 11);
}

#[test]
fn plan_sleep_returns_policy_and_cuts_rails() {
    let mut pm = manager();
    let plan = pm.plan_sleep().unwrap();
    assert_eq!(plan.timer_ms, 10_000);
    assert!(plan.wake_on_button);
    assert!(plan.rails.wants_on(Rail::Dcdc1));
    assert!(!plan.rails.wants_on(Rail::Aldo3));
    assert!(!plan.rails.wants_on(Rail::Aldo2));
    assert_eq!(pm.state(), PowerState::DeepSleep);
}

#[test]
fn reports_vbus_present() {
    let mut pm = PowerManager::new(FakePower::new(), Mode::Watch);
    assert!(!pm.is_vbus_present().unwrap());
    pm.pmic_mut().vbus = true;
    assert!(pm.is_vbus_present().unwrap());
}

#[test]
fn set_mode_applies_policy_and_rails() {
    let mut pm = PowerManager::new(FakePower::new(), Mode::Watch);
    pm.sync().unwrap();

    pm.set_mode(Mode::Desk).unwrap();
    assert_eq!(pm.mode(), Mode::Desk);
    assert_eq!(pm.state(), PowerState::Desk);
    assert_eq!(pm.policy().pixel, common::PixelMode::Gray8);
    assert!(pm.plan().wants_on(Rail::Cpusldo));

    pm.set_mode(Mode::Watch).unwrap();
    assert_eq!(pm.policy().pixel, common::PixelMode::Mono1);
    assert_eq!(pm.state(), PowerState::ActiveWatch);
}

#[test]
fn sync_reads_actual_state_and_sleep_cuts_extras() {
    let mut pm = PowerManager::new(FakePower::new(), Mode::Watch);
    // The PMIC powers up with DCDC2 on; the manager must not assume it is off.
    let dcdc2 = Rail::Dcdc2.index() as usize;
    pm.pmic_mut().enabled[dcdc2] = true;

    pm.sync().unwrap();
    assert!(pm.plan().wants_on(Rail::Dcdc2));

    pm.enter_sleep().unwrap();
    assert!(!pm.pmic_mut().enabled[dcdc2]);
}
