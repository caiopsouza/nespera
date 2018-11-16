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
    pub fn znco_adc(&mut self, a: u8, b: u8) {
        let value: u16 = a as u16 + b as u16;

        self.bits = (self.bits & !Self::Zero.bits & !Self::Negative.bits & !Self::Carry.bits & !Self::Overflow.bits)
            | ((((value as u8) == 0) as u8) * Self::Zero.bits)
            | ((value as u8) & Self::Negative.bits)
            | (((value & 0x0100u16) >> 8) as u8) // When adding, carry happens if bit 8 is set
            // Overflow happens when the sign of the addends is the same and differs from the sign of the sum
            | ((!(a ^ b) & (a ^ value as u8) & 0x80) >> 1)
        ;
    }

    // Set the Zero, Negative, Carry and Overflow flags according to the difference of the values passed
    pub fn znco_sbc(&mut self, a: u8, b: u8) {
        self.znco_adc(a, -(b as i8) as u8);
        self.toggle(Flags::Carry);
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

// Based on https://stackoverflow.com/a/8982549
#[cfg(test)]
mod znco {
    use super::*;

    #[cfg(test)]
    mod adc {
        use super::*;

        fn test(a: u8, b: u8, result: Flags) {
            let mut flags = Flags::empty();
            flags.znco_adc(a, b);
            assert_eq!(flags, result, "\nexpr: 0x{:02x?} + 0x{:02x?} = 0x{:02x?}", a, b, a as u16 + b as u16);
        }

        #[test]
        fn flags() { test(0x7f, 0x00, Flags::empty()); }

        #[test]
        fn flags_c() { test(0xff, 0x7f, Flags::Carry); }

        #[test]
        fn flags_z() { test(0x00, 0x00, Flags::Zero); }

        #[test]
        fn flags_zc() { test(0xff, 0x01, Flags::Zero | Flags::Carry); }

        #[test]
        fn flags_n() { test(0xff, 0x00, Flags::Negative); }

        #[test]
        fn flags_nc() { test(0xff, 0xff, Flags::Negative | Flags::Carry); }

        #[test]
        fn flags_co() { test(0xff, 0x80, Flags::Carry | Flags::Overflow); }

        #[test]
        fn flags_zco() { test(0x80, 0x80, Flags::Zero | Flags::Carry | Flags::Overflow); }

        #[test]
        fn flags_no() { test(0x7f, 0x7f, Flags::Negative | Flags::Overflow); }
    }

    #[cfg(test)]
    mod sbc {
        use super::*;
        use std::num::Wrapping;

        fn test(a: u8, b: u8, result: Flags) {
            let mut flags = Flags::empty();
            flags.znco_sbc(a, b);
            assert_eq!(flags, result, "\nexpr: 0x{0:02x?} - 0x{1:02x?} = 0x{0:02x?} + 0x{2:02x?} = 0x{3:02x?}", a, b, -(b as i8) as u8, (Wrapping(a as u16) - Wrapping(b as u16)).0);
        }

        #[test]
        fn flags() { test(0xff, 0xfe, Flags::empty()); }

        #[test]
        fn flags_c() { test(0x7e, 0xff, Flags::Carry); }

        #[test]
        fn flags_z() { test(0xff, 0xff, Flags::Zero); }

        #[test]
        fn flags_n() { test(0xff, 0x7f, Flags::Negative); }

        #[test]
        fn flags_nc() { test(0xfe, 0xff, Flags::Negative | Flags::Carry); }

        #[test]
        fn flags_o() { test(0xfe, 0x7f, Flags::Overflow); }

        #[test]
        fn flags_nco() { test(0x7f, 0xff, Flags::Negative | Flags::Carry | Flags::Overflow); }
    }
}
