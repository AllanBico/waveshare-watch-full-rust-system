//! AXP2101 output rails and their voltage encoding.
//!
//! Only the rails wired on this board are modeled. Voltage encoding is
//! piecewise per the datasheet; off-step and out-of-range values are rejected.

use super::error::RailError;
use super::regs::{DC_ONOFF_DVM, DC_PWM_BASE_BIT, DC_VOL_BASE, LDO_ONOFF0, LDO_VOL_BASE};
use super::units::Millivolts;

/// A managed AXP2101 output rail wired on this board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub enum Rail {
    /// DCDC1 (main 3.3 V rail).
    Dcdc1,
    /// DCDC2.
    Dcdc2,
    /// DCDC3.
    Dcdc3,
    /// DCDC4.
    Dcdc4,
    /// ALDO1.
    Aldo1,
    /// ALDO2.
    Aldo2,
    /// ALDO3.
    Aldo3,
    /// ALDO4.
    Aldo4,
    /// BLDO1.
    Bldo1,
    /// BLDO2.
    Bldo2,
    /// CPUSLDO.
    Cpusldo,
}

/// A contiguous linear voltage segment: `min + n * step` for `n` in `0..=count`.
struct Segment {
    min: u16,
    max: u16,
    step: u16,
    code_base: u8,
}

/// Register mapping and voltage segments for one rail.
struct RailSpec {
    voltage_reg: u8,
    enable_reg: u8,
    enable_bit: u8,
    segments: &'static [Segment],
}

const DCDC1_SEGMENTS: [Segment; 1] = [Segment {
    min: 1500,
    max: 3400,
    step: 100,
    code_base: 0,
}];
const DCDC2_SEGMENTS: [Segment; 2] = [
    Segment {
        min: 500,
        max: 1200,
        step: 10,
        code_base: 0,
    },
    Segment {
        min: 1220,
        max: 1540,
        step: 20,
        code_base: 71,
    },
];
const DCDC3_SEGMENTS: [Segment; 3] = [
    Segment {
        min: 500,
        max: 1200,
        step: 10,
        code_base: 0,
    },
    Segment {
        min: 1220,
        max: 1540,
        step: 20,
        code_base: 71,
    },
    Segment {
        min: 1600,
        max: 3400,
        step: 100,
        code_base: 88,
    },
];
const DCDC4_SEGMENTS: [Segment; 2] = [
    Segment {
        min: 500,
        max: 1200,
        step: 10,
        code_base: 0,
    },
    Segment {
        min: 1220,
        max: 1840,
        step: 20,
        code_base: 71,
    },
];
const ALDO_SEGMENTS: [Segment; 1] = [Segment {
    min: 500,
    max: 3500,
    step: 100,
    code_base: 0,
}];
const CPUSLDO_SEGMENTS: [Segment; 1] = [Segment {
    min: 500,
    max: 1400,
    step: 50,
    code_base: 0,
}];

const fn dcdc_spec(index: u8, segments: &'static [Segment]) -> RailSpec {
    RailSpec {
        voltage_reg: DC_VOL_BASE + index,
        enable_reg: DC_ONOFF_DVM,
        enable_bit: index,
        segments,
    }
}

const fn ldo_spec(index: u8, segments: &'static [Segment]) -> RailSpec {
    RailSpec {
        voltage_reg: LDO_VOL_BASE + index,
        enable_reg: LDO_ONOFF0,
        enable_bit: index,
        segments,
    }
}

impl Rail {
    const fn spec(self) -> RailSpec {
        match self {
            Self::Dcdc1 => dcdc_spec(0, &DCDC1_SEGMENTS),
            Self::Dcdc2 => dcdc_spec(1, &DCDC2_SEGMENTS),
            Self::Dcdc3 => dcdc_spec(2, &DCDC3_SEGMENTS),
            Self::Dcdc4 => dcdc_spec(3, &DCDC4_SEGMENTS),
            Self::Aldo1 => ldo_spec(0, &ALDO_SEGMENTS),
            Self::Aldo2 => ldo_spec(1, &ALDO_SEGMENTS),
            Self::Aldo3 => ldo_spec(2, &ALDO_SEGMENTS),
            Self::Aldo4 => ldo_spec(3, &ALDO_SEGMENTS),
            Self::Bldo1 => ldo_spec(4, &ALDO_SEGMENTS),
            Self::Bldo2 => ldo_spec(5, &ALDO_SEGMENTS),
            Self::Cpusldo => ldo_spec(6, &CPUSLDO_SEGMENTS),
        }
    }

    /// Register holding this rail's voltage setting.
    pub(super) const fn voltage_register(self) -> u8 {
        self.spec().voltage_reg
    }

    /// Register holding this rail's enable bit.
    pub(super) const fn enable_register(self) -> u8 {
        self.spec().enable_reg
    }

    /// Enable-bit position within [`Self::enable_register`].
    pub(super) const fn enable_bit(self) -> u8 {
        self.spec().enable_bit
    }

    /// Forced-PWM bit for a DCDC rail, or `None` for LDO rails.
    pub(super) const fn dcdc_pwm_bit(self) -> Option<u8> {
        match self {
            Self::Dcdc1 => Some(DC_PWM_BASE_BIT),
            Self::Dcdc2 => Some(DC_PWM_BASE_BIT + 1),
            Self::Dcdc3 => Some(DC_PWM_BASE_BIT + 2),
            Self::Dcdc4 => Some(DC_PWM_BASE_BIT + 3),
            _ => None,
        }
    }

    /// Encodes a millivolt setting into the 7-bit register value.
    pub(super) fn encode(self, millivolts: Millivolts) -> Result<u8, RailError> {
        let mv = millivolts.value();
        for segment in self.spec().segments {
            if mv < segment.min || mv > segment.max {
                continue;
            }
            let delta = mv - segment.min;
            if !delta.is_multiple_of(segment.step) {
                return Err(RailError::OffStep);
            }
            return Ok(segment.code_base + (delta / segment.step) as u8);
        }
        Err(RailError::OutOfRange)
    }

    /// Decodes a 7-bit register value back into millivolts.
    pub(super) fn decode(self, code: u8) -> Result<Millivolts, RailError> {
        let code = code & 0x7F;
        for segment in self.spec().segments {
            let count = ((segment.max - segment.min) / segment.step) as u8;
            let end = segment.code_base + count;
            if code >= segment.code_base && code <= end {
                let mv = segment.min + (code - segment.code_base) as u16 * segment.step;
                return Ok(Millivolts::new(mv));
            }
        }
        Err(RailError::OutOfRange)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(rail: Rail, mv: u16) {
        let code = rail.encode(Millivolts::new(mv)).expect("encodes");
        assert_eq!(rail.decode(code).expect("decodes").value(), mv);
    }

    #[test]
    fn linear_rails_roundtrip() {
        roundtrip(Rail::Dcdc1, 3300);
        roundtrip(Rail::Aldo1, 3300);
        roundtrip(Rail::Bldo2, 2800);
        roundtrip(Rail::Cpusldo, 1200);
    }

    #[test]
    fn piecewise_rails_roundtrip() {
        roundtrip(Rail::Dcdc2, 900);
        roundtrip(Rail::Dcdc2, 1220);
        roundtrip(Rail::Dcdc3, 1800);
        roundtrip(Rail::Dcdc4, 1800);
    }

    #[test]
    fn known_codes_decode() {
        assert_eq!(Rail::Dcdc2.decode(71).unwrap().value(), 1220);
        assert_eq!(Rail::Dcdc3.decode(88).unwrap().value(), 1600);
        assert_eq!(Rail::Dcdc1.decode(13).unwrap().value(), 2800);
    }

    #[test]
    fn off_step_is_rejected() {
        assert_eq!(
            Rail::Dcdc1.encode(Millivolts::new(1505)),
            Err(RailError::OffStep)
        );
    }

    #[test]
    fn out_of_range_is_rejected() {
        assert_eq!(
            Rail::Dcdc1.encode(Millivolts::new(1000)),
            Err(RailError::OutOfRange)
        );
    }

    #[test]
    fn rail_register_mapping() {
        assert_eq!(Rail::Dcdc1.voltage_register(), 0x82);
        assert_eq!(Rail::Cpusldo.voltage_register(), 0x98);
        assert_eq!(Rail::Bldo2.enable_bit(), 5);
        assert_eq!(Rail::Aldo1.dcdc_pwm_bit(), None);
        assert_eq!(Rail::Dcdc4.dcdc_pwm_bit(), Some(5));
    }
}
