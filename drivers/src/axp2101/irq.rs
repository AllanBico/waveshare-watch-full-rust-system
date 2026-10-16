//! AXP2101 interrupt mask.

/// A set of AXP2101 interrupt bits (24 bits across three registers).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IrqMask(u32);

impl IrqMask {
    /// No interrupts.
    pub const EMPTY: Self = Self(0);
    /// Battery under-temperature in work mode.
    pub const BAT_UNDER_TEMP: Self = Self(1);
    /// Battery over-temperature in work mode.
    pub const BAT_OVER_TEMP: Self = Self(1 << 1);
    /// Battery under-temperature while charging.
    pub const BAT_CHG_UNDER_TEMP: Self = Self(1 << 2);
    /// Battery over-temperature while charging.
    pub const BAT_CHG_OVER_TEMP: Self = Self(1 << 3);
    /// Gauge reports a new state of charge.
    pub const GAUGE_NEW_SOC: Self = Self(1 << 4);
    /// Gauge watchdog timeout.
    pub const WDT_TIMEOUT: Self = Self(1 << 5);
    /// State of charge fell to warning level 1.
    pub const LOW_BATTERY1: Self = Self(1 << 6);
    /// State of charge fell to warning level 2.
    pub const LOW_BATTERY2: Self = Self(1 << 7);
    /// PWR button positive edge.
    pub const PKEY_POSITIVE: Self = Self(1 << 8);
    /// PWR button negative edge.
    pub const PKEY_NEGATIVE: Self = Self(1 << 9);
    /// PWR button long press.
    pub const PKEY_LONG: Self = Self(1 << 10);
    /// PWR button short press.
    pub const PKEY_SHORT: Self = Self(1 << 11);
    /// Battery removed.
    pub const BAT_REMOVE: Self = Self(1 << 12);
    /// Battery inserted.
    pub const BAT_INSERT: Self = Self(1 << 13);
    /// VBUS removed.
    pub const VBUS_REMOVE: Self = Self(1 << 14);
    /// VBUS inserted.
    pub const VBUS_INSERT: Self = Self(1 << 15);
    /// Battery over-voltage.
    pub const BAT_OVER_VOLTAGE: Self = Self(1 << 16);
    /// Charger safety timer expired.
    pub const CHARGER_TIMER: Self = Self(1 << 17);
    /// Die over-temperature.
    pub const DIE_OVER_TEMP: Self = Self(1 << 18);
    /// Charging started.
    pub const CHARGE_START: Self = Self(1 << 19);
    /// Charging done.
    pub const CHARGE_DONE: Self = Self(1 << 20);
    /// BATFET over-current.
    pub const BATFET_OVER_CURRENT: Self = Self(1 << 21);
    /// LDO over-current.
    pub const LDO_OVER_CURRENT: Self = Self(1 << 22);
    /// Watchdog expired.
    pub const WDT_EXPIRE: Self = Self(1 << 23);

    /// The raw 24-bit value.
    pub const fn bits(self) -> u32 {
        self.0
    }

    /// Whether no bits are set.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Combines two masks.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Builds a mask from the three IRQ status bytes.
    pub(super) const fn from_bytes(bytes: [u8; 3]) -> Self {
        Self(bytes[0] as u32 | (bytes[1] as u32) << 8 | (bytes[2] as u32) << 16)
    }
}

impl core::ops::BitOr for IrqMask {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn irq_bits_combine() {
        let mask = IrqMask::VBUS_INSERT | IrqMask::CHARGE_DONE;
        assert_eq!(mask.bits(), (1 << 15) | (1 << 20));
        assert!(!mask.is_empty());
        assert!(IrqMask::EMPTY.is_empty());
    }

    #[test]
    fn from_bytes_maps_registers() {
        let mask = IrqMask::from_bytes([0, 1 << 7, 1 << 4]);
        assert_eq!(
            mask.bits(),
            IrqMask::VBUS_INSERT.bits() | IrqMask::CHARGE_DONE.bits()
        );
    }
}
