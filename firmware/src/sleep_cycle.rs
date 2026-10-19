//! Pre-sleep checklist and the final deep-sleep call.
//!
//! `power` owns the rail plan; this module owns the ordered subsystem and SoC
//! steps that must run before that plan is applied. Watch mode reaches it after
//! the awake budget expires; desk reaches it after a live switch to watch.

use core::fmt::Debug;

use esp_hal::gpio::{RtcPin, RtcPinWithResistors};
use esp_hal::peripherals::{GPIO21, LPWR};
use esp_hal::rtc_cntl::sleep::{Ext1WakeupSource, TimerWakeupSource, WakeupLevel};
use esp_hal::rtc_cntl::Rtc;
use esp_println::println;
use hal::Display;
use power::PowerManager;

use alarms::AlarmStore;
use common::{Error, Timestamp, WakeOnMotion};
use settings::{BootPolicy, Settings};

use crate::alarm_boot::Backend;
use crate::boot::{self, WOM_BLANKING_SAMPLES};
use crate::retained;

/// Everything `main` owns that the pre-sleep checklist needs.
pub struct SleepInputs<'a, D, P, R> {
    /// The panel, when bring-up succeeded.
    pub display: &'a mut Option<D>,
    /// The touch controller, when desk initialized it.
    pub touch: &'a mut Option<boot::Touch>,
    /// The IMU.
    pub imu: &'a mut boot::Imu,
    /// Stable IMU bring-up error, when init failed.
    pub imu_error: Option<u16>,
    /// Power manager and its budget.
    pub power: &'a mut PowerManager<P>,
    /// Current settings snapshot.
    pub settings: &'a Settings,
    /// PWR button pin.
    pub pwr_button: &'a mut R,
    /// Stored alarms, used to schedule the next RTC wake.
    pub alarms: &'a mut AlarmStore<Backend>,
    /// Wall-clock time at this sleep.
    pub now: Timestamp,
    /// IMU interrupt pin.
    pub imu_int: GPIO21<'static>,
    /// Low-power peripheral token for the RTC.
    pub lpw: LPWR<'static>,
    /// Whether serial logging is enabled.
    pub log: bool,
}

/// Runs the pre-sleep checklist, then enters deep sleep.
pub async fn enter<D, P, R>(input: SleepInputs<'_, D, P, R>) -> !
where
    D: Display,
    D::Error: Debug,
    P: hal::Power,
    R: RtcPinWithResistors,
{
    // Re-derive after any live switch so pre-sleep wake settings are current.
    let policy = BootPolicy::from_settings(input.settings);
    if input.imu_error.is_none() && policy.wake_on_motion {
        let config = WakeOnMotion::new(policy.motion_threshold_mg, WOM_BLANKING_SAMPLES);
        if let Err(error) = input.imu.configure_wake_on_motion(config) {
            retained::record_error(Error::HardwareFailure.code());
            if input.log {
                println!("[imu] wake-on-motion failed: {}", error);
            }
        }
    }
    if let Some(touch) = input.touch.as_mut() {
        if let Err(error) = touch.sleep() {
            retained::record_error(Error::HardwareFailure.code());
            if input.log {
                println!("[tp] sleep failed: {}", error);
            }
        }
    }

    // Sleep the panel before `power` cuts its enable rail (ALDO2).
    if let Some(display) = input.display.as_mut() {
        if let Err(error) = display.display_off() {
            retained::record_error(Error::HardwareFailure.code());
            if input.log {
                println!("[lcd] sleep failed: {:?}", error);
            }
        }
    }

    let timer_ms = input.alarms.next_wake_in_ms(input.now);
    let plan = match input.power.plan_sleep(timer_ms) {
        Ok(plan) => plan,
        Err(error) => {
            retained::record_error(error.to_common().code());
            if input.log {
                println!("[pm] plan_sleep failed: {}", error.to_common());
            }
            crate::runtime::halt().await
        }
    };
    if input.log {
        match plan.timer_ms {
            Some(timer_ms) => println!("[pm] sleeping {} ms", timer_ms),
            None => println!("[pm] sleeping until wake source"),
        }
    }

    let mut rtc = Rtc::new(input.lpw);
    // The PWR button needs the RTC pull-up: the external network alone did not
    // produce a reliable high when tested with the digital input.
    input.pwr_button.rtcio_pullup(true);
    input.pwr_button.rtcio_pulldown(false);
    let mut imu_int = input.imu_int;
    let mut wake_pins: [&mut dyn RtcPin; 2] = [input.pwr_button, &mut imu_int];
    let external = Ext1WakeupSource::new(&mut wake_pins, WakeupLevel::High);
    match plan.timer_ms {
        Some(timer_ms) => {
            let timer =
                TimerWakeupSource::new(core::time::Duration::from_millis(u64::from(timer_ms)));
            rtc.sleep_deep(&[&timer, &external])
        }
        None => rtc.sleep_deep(&[&external]),
    }
}
