use std::ops;

use crate::utils::bits;

// Bit for each flag
pub const CARRY: Flags = Flags(0b0000_0001);
pub const ZERO: Flags = Flags(0b0000_0010);
pub const INTERRUPT_DISABLE: Flags = Flags(0b0000_0100);
pub const DECIMAL_MODE: Flags = Flags(0b0000_1000);
pub const BREAK_COMMAND: Flags = Flags(0b0001_0000);
pub const UNUSED: Flags = Flags(0b0010_0000);
pub const OVERFLOW: Flags = Flags(0b0100_0000);
pub const NEGATIVE: Flags = Flags(0b1000_0000);

pub const LEAST_BIT: u8 = 0b1000_0000;

// Flags for the P register
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub struct Flags(pub u8);

impl Flags {
    pub fn as_u8(self) -> u8 { self.0 }

    pub fn change(&mut self, other: Self, condition: bool) {
        self.0 = if condition { self.0 | other.0 } else { self.0 & !other.0 }
    }

    pub fn copy(&mut self, other: Self, mask: Self) {
        self.0 = bits::copy(self.0, other.0, mask.0)
    }

    pub fn contains(self, flags: Self) -> bool {
        (self.0 & flags.0) == flags.0
    }

    // Set and clear
    pub fn set(&mut self, flags: Self) { self.copy(flags, flags) }
    pub fn clear(&mut self, flags: Self) { self.copy(!flags, flags) }

    pub fn toggle(&mut self, other: Self) {
        if self.contains(other) {
            self.clear(other)
        } else {
            self.set(other)
        }
    }

    // Getter
    pub fn get_carry(self) -> bool { self.contains(CARRY) }
    pub fn get_zero(self) -> bool { self.contains(ZERO) }
    pub fn get_interrupt_disable(self) -> bool { self.contains(INTERRUPT_DISABLE) }
    pub fn get_decimal_mode(self) -> bool { self.contains(DECIMAL_MODE) }
    pub fn get_break_command(self) -> bool { self.contains(BREAK_COMMAND) }
    pub fn get_unused(self) -> bool { self.contains(UNUSED) }
    pub fn get_overflow(self) -> bool { self.contains(OVERFLOW) }
    pub fn get_negative(self) -> bool { self.contains(NEGATIVE) }

    pub fn change_zero_negative(&mut self, value: u8) {
        self.change(ZERO, value == 0);
        self.change(NEGATIVE, (value & LEAST_BIT) != 0);
    }

    // Set the Zero, Negative and Carry flag based on a comparison
    pub fn change_cmp(&mut self, value: u8, other: u8) {
        self.change(ZERO, value == other);
        self.change(CARRY, value >= other);

        // Negative has the same bit as the 7th of the difference
        let diff = value.wrapping_sub(other);
        self.change(NEGATIVE, (diff & 0b1000_0000) != 0);
    }
}

impl From<u8> for Flags {
    fn from(data: u8) -> Self { Self(data) }
}

impl Into<u8> for Flags {
    fn into(self) -> u8 { self.0 }
}

impl ops::Not for Flags {
    type Output = Self;

    fn not(self) -> <Self as ops::Not>::Output { Self(!self.0) }
}

impl ops::BitAnd for Flags {
    type Output = Self;

    fn bitand(self, rhs: Self) -> <Self as ops::BitAnd<Self>>::Output {
        Self(self.0 & rhs.0)
    }
}

impl ops::BitOr for Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> <Self as ops::BitAnd<Self>>::Output {
        Self(self.0 | rhs.0)
    }
}

impl ops::BitAndAssign for Flags {
    fn bitand_assign(&mut self, rhs: Self) { self.0 &= rhs.0 }
}

impl ops::BitOrAssign for Flags {
    fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0 }
}
