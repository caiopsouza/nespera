use std::convert::From;

// Flags for the P register
bitflags! {
    pub struct Flags: u8 {
        const Carry = 0b00000001;
        const Zero = 0b00000010;
        const InterruptDisable = 0b00000100;
        const DecimalMode = 0b00001000;
        const BreakCommand = 0b00010000;
        const Unused = 0b00100000;
        const Overflow = 0b01000000;
        const Negative = 0b10000000;
    }
}

impl Flags {
    // Set the Zero and Negative flags according to the value passed
    pub fn zn(&mut self, value: u8) {
        self.bits = (self.bits & !Self::Zero.bits & !Self::Negative.bits)
            | (((value == 0) as u8) * Self::Zero.bits)
            | (value & Self::Negative.bits);
    }

    // Set the Zero, Negative and Overflow flags according to the value passed using "Bit Test" rules
    pub fn zno_bit_test(&mut self, value: u8) {
        self.bits = (self.bits & !Self::Zero.bits & !Self::Negative.bits & !Self::Overflow.bits)
            | (((value == 0) as u8) * Self::Zero.bits)
            | (value & (Self::Negative.bits | Self::Overflow.bits));
    }

    // Set the Zero, Negative, Carry and Overflow flags according to the sum of the values passed
    pub fn znco(&mut self, a: u8, b: u8) {
        let value: u16 = a as u16 + b as u16;

        self.bits = (self.bits & !Self::Zero.bits & !Self::Negative.bits & !Self::Carry.bits & !Self::Overflow.bits)
            | ((((value as u8) == 0) as u8) * Self::Zero.bits)
            | ((value as u8) & Self::Negative.bits)
            | (((value & 0x0100u16) >> 8) as u8) // Carry happens if bit 8 is set
            // Overflow happens when the sign of the addends is the same and differs from the sign of the sum
            | ((!(a ^ b) & (a ^ value as u8) & 0x80) >> 1)
        ;
    }
}

impl From<u8> for Flags {
    fn from(bits: u8) -> Self { Self { bits } }
}

#[cfg(test)]
mod zn {
    use super::*;

    fn test(value: u8, mut initial: Flags, result: Flags) {
        initial.zn(value);
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
mod zno_bit_test {
    use super::*;

    fn test(value: u8, mut initial: Flags, result: Flags) {
        initial.zno_bit_test(value);
        assert_eq!(initial, result, "\nvalue: {:x?}", value);
    }

    #[test]
    fn zero() { test(0, Flags::empty(), Flags::Zero); }

    #[test]
    fn positive() { test(63, Flags::all(), !Flags::Negative & !Flags::Overflow & !Flags::Zero); }

    #[test]
    fn positive_overflow() { test(127, Flags::all(), !Flags::Negative & !Flags::Zero); }

    #[test]
    fn negative() { test(128, Flags::empty(), Flags::Negative); }

    #[test]
    fn negative_overflow() { test(204, Flags::empty(), Flags::Negative | Flags::Overflow); }
}

#[cfg(test)]
mod znco {
    use super::*;

    fn test(a: u8, b: u8, result: Flags) {
        let mut flags = Flags::empty();
        flags.znco(a, b);
        assert_eq!(flags, result, "\nvalue: {:x?}", a as u16 + b as u16);
    }

    #[test]
    fn sum_0x00_0x00() { test(0x00u8, 0x00u8, Flags::Zero); }

    #[test]
    fn sum_0xa0_0xa0() { test(0xa0u8, 0xa0u8, Flags::Carry | Flags::Overflow); }

    #[test]
    fn sum_0x90_0x90() { test(0x90u8, 0x90u8, Flags::Carry | Flags::Overflow); }

    #[test]
    fn sum_0x80_0x80() { test(0x80u8, 0x80u8, Flags::Zero | Flags::Carry | Flags::Overflow); }

    #[test]
    fn sum_0x00_0x80() { test(0x00u8, 0x80u8, Flags::Negative); }

    #[test]
    fn sum_0xe0_0xe0() { test(0xe0u8, 0xe0u8, Flags::Negative | Flags::Carry); }
}
