//! AXP2101 rail voltage encoding.
//!
//! Maps the shared [`Rail`] contract onto AXP2101 voltage/enable registers.
//! Voltage encoding is piecewise per the datasheet; off-step and out-of-range
//! values are rejected.

use common::Millivolts;
use hal::Rail;

use super::error::RailError;
use super::regs::{DC_ONOFF_DVM, DC_PWM_BASE_BIT, DC_VOL_BASE, LDO_ONOFF0, LDO_VOL_BASE};

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

fn spec(rail: Rail) -> RailSpec {
    match rail {
        Rail::Dcdc1 => dcdc_spec(0, &DCDC1_SEGMENTS),
        Rail::Dcdc2 => dcdc_spec(1, &DCDC2_SEGMENTS),
        Rail::Dcdc3 => dcdc_spec(2, &DCDC3_SEGMENTS),
        Rail::Dcdc4 => dcdc_spec(3, &DCDC4_SEGMENTS),
        Rail::Aldo1 => ldo_spec(0, &ALDO_SEGMENTS),
        Rail::Aldo2 => ldo_spec(1, &ALDO_SEGMENTS),
        Rail::Aldo3 => ldo_spec(2, &ALDO_SEGMENTS),
        Rail::Aldo4 => ldo_spec(3, &ALDO_SEGMENTS),
        Rail::Bldo1 => ldo_spec(4, &ALDO_SEGMENTS),
        Rail::Bldo2 => ldo_spec(5, &ALDO_SEGMENTS),
        Rail::Cpusldo => ldo_spec(6, &CPUSLDO_SEGMENTS),
    }
}

/// Register holding `rail`'s voltage setting.
pub(super) fn voltage_register(rail: Rail) -> u8 {
    spec(rail).voltage_reg
}

/// Register holding `rail`'s enable bit.
pub(super) fn enable_register(rail: Rail) -> u8 {
    spec(rail).enable_reg
}

/// Enable-bit position within [`enable_register`].
pub(super) fn enable_bit(rail: Rail) -> u8 {
    spec(rail).enable_bit
}

/// Forced-PWM bit for a DCDC rail, or `None` for LDO rails.
pub(super) fn dcdc_pwm_bit(rail: Rail) -> Option<u8> {
    match rail {
        Rail::Dcdc1 => Some(DC_PWM_BASE_BIT),
        Rail::Dcdc2 => Some(DC_PWM_BASE_BIT + 1),
        Rail::Dcdc3 => Some(DC_PWM_BASE_BIT + 2),
        Rail::Dcdc4 => Some(DC_PWM_BASE_BIT + 3),
        _ => None,
    }
}

/// Encodes a millivolt setting into the 7-bit register value.
pub(super) fn encode(rail: Rail, millivolts: Millivolts) -> Result<u8, RailError> {
    let mv = millivolts.value();
    for segment in spec(rail).segments {
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
pub(super) fn decode(rail: Rail, code: u8) -> Result<Millivolts, RailError> {
    let code = code & 0x7F;
    for segment in spec(rail).segments {
        let count = ((segment.max - segment.min) / segment.step) as u8;
        let end = segment.code_base + count;
        if code >= segment.code_base && code <= end {
            let mv = segment.min + (code - segment.code_base) as u16 * segment.step;
            return Ok(Millivolts::new(mv));
        }
    }
    Err(RailError::OutOfRange)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(rail: Rail, mv: u16) {
        let code = encode(rail, Millivolts::new(mv)).expect("encodes");
        assert_eq!(decode(rail, code).expect("decodes").value(), mv);
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
        assert_eq!(decode(Rail::Dcdc2, 71).unwrap().value(), 1220);
        assert_eq!(decode(Rail::Dcdc3, 88).unwrap().value(), 1600);
        assert_eq!(decode(Rail::Dcdc1, 13).unwrap().value(), 2800);
    }

    #[test]
    fn off_step_is_rejected() {
        assert_eq!(
            encode(Rail::Dcdc1, Millivolts::new(1505)),
            Err(RailError::OffStep)
        );
    }

    #[test]
    fn out_of_range_is_rejected() {
        assert_eq!(
            encode(Rail::Dcdc1, Millivolts::new(1000)),
            Err(RailError::OutOfRange)
        );
    }

    #[test]
    fn register_mapping() {
        assert_eq!(voltage_register(Rail::Dcdc1), 0x82);
        assert_eq!(voltage_register(Rail::Cpusldo), 0x98);
        assert_eq!(enable_bit(Rail::Bldo2), 5);
        assert_eq!(dcdc_pwm_bit(Rail::Aldo1), None);
        assert_eq!(dcdc_pwm_bit(Rail::Dcdc4), Some(5));
    }
}
