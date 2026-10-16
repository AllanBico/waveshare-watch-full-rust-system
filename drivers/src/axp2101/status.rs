//! AXP2101 status, snapshot, and interrupt types.

use common::{Millivolts, Percent};

use super::regs::{
    STATUS1_BAT_PRESENT, STATUS1_VBUS_GOOD, STATUS2_CHARGING, STATUS2_CHG_SHIFT, STATUS2_POWER_ON,
    STATUS2_VBUS_IN,
};

/// Charger state machine position (STATUS2 bits 2:0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ChargeState {
    /// Trickle charge.
    Tri,
    /// Pre-charge.
    Pre,
    /// Constant current.
    Cc,
    /// Constant voltage.
    Cv,
    /// Charge complete.
    Done,
    /// Not charging.
    Stop,
}

impl ChargeState {
    /// Decodes the low three bits of STATUS2.
    pub const fn from_bits(bits: u8) -> Option<Self> {
        match bits & 0x07 {
            0 => Some(Self::Tri),
            1 => Some(Self::Pre),
            2 => Some(Self::Cc),
            3 => Some(Self::Cv),
            4 => Some(Self::Done),
            5 => Some(Self::Stop),
            _ => None,
        }
    }
}

/// Decoded power status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Status {
    /// VBUS is present and good.
    pub vbus_good: bool,
    /// VBUS input is active.
    pub vbus_in: bool,
    /// A battery is connected.
    pub battery_present: bool,
    /// The charger is charging.
    pub charging: bool,
    /// The PMIC is powered on.
    pub power_on: bool,
    /// Charger state, or `None` if the field is reserved.
    pub charge_state: Option<ChargeState>,
}

impl Status {
    /// Decodes STATUS1 and STATUS2 register bytes.
    pub const fn from_registers(status1: u8, status2: u8) -> Self {
        Self {
            vbus_good: status1 & STATUS1_VBUS_GOOD != 0,
            battery_present: status1 & STATUS1_BAT_PRESENT != 0,
            vbus_in: (status2 & STATUS2_VBUS_IN == 0) && (status1 & STATUS1_VBUS_GOOD != 0),
            charging: (status2 >> STATUS2_CHG_SHIFT) == STATUS2_CHARGING,
            power_on: status2 & STATUS2_POWER_ON != 0,
            charge_state: ChargeState::from_bits(status2),
        }
    }
}

/// A consistent sample of the PMIC's voltage and charge state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PowerSnapshot {
    /// Battery voltage.
    pub battery: Millivolts,
    /// VBUS voltage (0 when not present).
    pub vbus: Millivolts,
    /// System (VSYS) voltage.
    pub system: Millivolts,
    /// Battery state of charge.
    pub percent: Percent,
    /// Decoded status.
    pub status: Status,
}

/// The minimal power reading taken on the deep-sleep wake path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct WakePowerState {
    /// Battery voltage.
    pub battery: Millivolts,
    /// VBUS (USB) is present.
    pub vbus_present: bool,
}

/// A measurable ADC channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum AdcChannel {
    /// Battery voltage.
    BatteryVoltage,
    /// TS-pin (battery NTC) voltage.
    TsPin,
    /// VBUS voltage.
    VbusVoltage,
    /// System voltage.
    SystemVoltage,
    /// Die temperature.
    DieTemp,
}

impl AdcChannel {
    /// The ADC-enable register bit for this channel.
    pub(super) const fn bit(self) -> u8 {
        match self {
            Self::BatteryVoltage => 0,
            Self::TsPin => 1,
            Self::VbusVoltage => 2,
            Self::SystemVoltage => 3,
            Self::DieTemp => 4,
        }
    }
}

/// Charger constant-voltage target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum ChargeTarget {
    /// 4.0 V.
    V4_0,
    /// 4.1 V.
    V4_1,
    /// 4.2 V.
    V4_2,
    /// 4.35 V.
    V4_35,
    /// 4.4 V.
    V4_4,
}

impl ChargeTarget {
    /// The 3-bit register value for this target.
    pub(super) const fn bits(self) -> u8 {
        match self {
            Self::V4_0 => 0,
            Self::V4_1 => 1,
            Self::V4_2 => 2,
            Self::V4_35 => 3,
            Self::V4_4 => 4,
        }
    }
}

/// DCDC switching mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DcdcMode {
    /// Automatic PFM/PWM (efficient at light load).
    Auto,
    /// Forced PWM (lower ripple, higher light-load current).
    Pwm,
}

/// DVM output slew ramp.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DvmRamp {
    /// 15.625 microseconds per step.
    Us15_625,
    /// 31.250 microseconds per step.
    Us31_250,
}

/// Behavior of a long PWR-button press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LongPressAction {
    /// Restart the PMIC/SoC.
    Restart,
    /// Power off.
    PowerOff,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_decodes_charging() {
        let status = Status::from_registers(0x28, 0x20 | 0x02);
        assert!(status.vbus_good);
        assert!(status.battery_present);
        assert!(status.charging);
        assert_eq!(status.charge_state, Some(ChargeState::Cc));
    }

    #[test]
    fn charging_only_when_field_is_one() {
        let discharging = Status::from_registers(0x08, 0x02 << 5);
        assert!(!discharging.charging);
    }

    #[test]
    fn reserved_charge_state_is_none() {
        assert_eq!(ChargeState::from_bits(6), None);
    }
}
