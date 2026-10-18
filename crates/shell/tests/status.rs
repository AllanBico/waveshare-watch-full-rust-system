mod support;

use common::{Mode, Percent, Region, Timestamp};
use shell::Shell;
use support::{Log, Registry, HOME};

#[test]
fn shell_owns_the_status_model() {
    let mut shell = Shell::new(
        Registry::with_battery(Log::default(), Percent::new(42)),
        HOME,
        Mode::Watch,
        Region::full(410, 502),
    );
    shell.start().unwrap();
    assert_eq!(shell.status().battery(), Percent::new(42));
    assert!(shell.status().is_charging());

    let now = Timestamp::from_secs(1_700_000_000);
    shell.set_time(now);
    assert_eq!(shell.status().clock(), now);

    shell.set_battery(Percent::new(17), false);
    assert_eq!(shell.status().battery(), Percent::new(17));
    assert!(!shell.status().is_charging());
}
