use std::convert::From;

// Flags for the P register
bitflags! {
    pub struct Flags: u8 {
        const Carry =            0b00000001;
        const Zero =             0b00000010;
        const InterruptDisable = 0b00000100;
        const DecimalMode =      0b00001000;
        const BreakCommand =     0b00010000;
        const Unused =           0b00100000;
        const Overflow =         0b01000000;
        const Negative =         0b10000000;
    }
}

impl Flags {
    // Copy the flags for the specified byte
    pub fn copy(&mut self, flags: Flags, value: u8) {
        self.bits = (self.bits & !flags.bits) | (value & flags.bits);
    }

    pub fn change_zero(&mut self, value: u8) {
        self.set(Flags::Zero, value == 0);
    }

    pub fn change_negative(&mut self, value: u8) {
        self.set(Self::Negative, (value & 0b10000000) != 0);
    }

    pub fn change_zero_and_negative(&mut self, value: u8) {
        self.change_zero(value);
        self.change_negative(value);
    }

    // Set the Zero, Negative and Carry flag based on a comparison
    pub fn change_cmp(&mut self, value: u8, other: u8) {
        self.set(Self::Zero, value == other);
        self.set(Self::Carry, value >= other);
        self.set(Self::Negative, (value as i8) < (other as i8));
    }

    // Set the Zero, Negative and Carry Flag based on the left shift of the value passed
    pub fn change_left_shift(&mut self, original_value: u8) {
        self.set(Flags::Carry, (original_value & 0b1000000) != 0);
        self.change_zero_and_negative(original_value << 1);
    }

    // Set the Zero, Negative and Carry Flag based on the right shift of the value passed
    pub fn change_right_shift(&mut self, original_value: u8) {
        self.set(Flags::Carry, (original_value & 0b0000001) != 0);
        self.change_zero_and_negative(original_value >> 1);
    }

    // Set the Zero, Negative and Carry Flag based on the right rotation of the value passed
    pub fn change_right_rotate(&mut self, original_value: u8) {
        let carry = (self.bits & Self::Carry.bits) << 7;
        self.set(Flags::Carry, (original_value & 0b0000001) != 0);
        self.change_zero_and_negative((original_value >> 1) | carry);
    }
}

impl From<u8> for Flags {
    fn from(bits: u8) -> Self { Self { bits } }
}

#[cfg(test)]
mod zero_negative {
    use super::*;

    fn test(value: u8, mut initial: Flags, result: Flags) {
        initial.change_zero_and_negative(value);
        assert_eq!(initial, result, "\nvalue: {:x?}", value);
    }

    #[test]
    fn zero() { test(0, Flags::empty(), Flags::Zero); }

    #[test]
    fn positive() { test(127, Flags::all(), !Flags::Negative & !Flags::Zero); }

    #[test]
    fn negative() { test(128, Flags::empty(), Flags::Negative); }
}

#[cfg(test)]
mod change_cmp {
    use super::*;

    fn test(a: u8, b: u8, result: Flags) {
        let mut flags = Flags::empty();
        flags.change_cmp(a, b);
        assert_eq!(flags, result, "\nexpr: 0x{:02x?} + 0x{:02x?}", a, b);
    }

    #[test]
    fn carry() { test(0xff, 0xfe, Flags::Carry); }

    #[test]
    fn zero_carry() { test(0xff, 0xff, Flags::Zero | Flags::Carry); }

    #[test]
    fn negative() { test(0x80, 0xff, Flags::Negative); }
}