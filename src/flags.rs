use std::convert::From;
use std::num::Wrapping;

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
    pub fn znv_bit_test(&mut self, value: u8) {
        self.bits = (self.bits & !Self::Zero.bits & !Self::Negative.bits & !Self::Overflow.bits)
            | (((value == 0) as u8) * Self::Zero.bits)
            | (value & (Self::Negative.bits | Self::Overflow.bits));
    }

    // Set the Zero, Negative, Carry and Overflow flags according to the sum of the values passed
    pub fn zncv_adc(&mut self, a: u8, b: u8) {
        let value: u16 = a as u16 + b as u16;
        self.zn(value as u8);
        self.bits = (self.bits & !Self::Carry.bits & !Self::Overflow.bits)
            | (((value & 0x0100u16) >> 8) as u8) // When adding, carry happens if bit 8 is set
            // Overflow happens when the sign of the addends is the same and differs from the sign of the sum
            | ((!(a ^ b) & (a ^ value as u8) & 0x80) >> 1)
        ;
    }

    // Set the Zero, Negative, Carry and Overflow flags according to the difference of the values passed
    pub fn zncv_sbc(&mut self, a: u8, b: u8) {
        self.zncv_adc(a, (-Wrapping(b as i8)).0 as u8);
        self.toggle(Flags::Carry); // Carry when subtraction is the opposition of adding
    }

    // Set the Zero, Negative and Carry flag based on a comparison
    pub fn znc_cmp(&mut self, value: u8, other: u8) {
        let diff: i16 = (value as i8) as i16 - (other as i8) as i16;

        self.bits = (self.bits & !Self::Zero.bits & !Self::Negative.bits & !Self::Carry.bits)
            | (((diff == 0) as u8) * Self::Zero.bits)
            | (((diff >= 0) as u8) * Self::Carry.bits)
            | (((diff < 0) as u8) * Self::Negative.bits)
    }

    // Set the Zero, Negative and Carry Flag based on the left shift of the value passed
    pub fn znc_left_shift(&mut self, original_value: u8) {
        self.bits = (self.bits & !Self::Carry.bits) | ((original_value & 0b1000000) >> 6);
        self.zn(original_value << 1);
    }

    // Set the Zero, Negative and Carry Flag based on the right shift of the value passed
    pub fn znc_right_shift(&mut self, original_value: u8) {
        self.bits = (self.bits & !Self::Carry.bits) | (original_value & 0b0000001);
        self.zn(original_value >> 1);
    }

    // Set the Zero, Negative and Carry Flag based on the right rotation of the value passed
    pub fn znc_right_rotate(&mut self, original_value: u8) {
        let carry = (self.bits & Self::Carry.bits) << 7;
        self.bits = (self.bits & !Self::Carry.bits) | (original_value & 0b0000001);
        self.zn((original_value >> 1) | carry);
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
        initial.znv_bit_test(value);
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
            flags.zncv_adc(a, b);
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
        fn flags_cv() { test(0xff, 0x80, Flags::Carry | Flags::Overflow); }

        #[test]
        fn flags_zcv() { test(0x80, 0x80, Flags::Zero | Flags::Carry | Flags::Overflow); }

        #[test]
        fn flags_nv() { test(0x7f, 0x7f, Flags::Negative | Flags::Overflow); }
    }

    #[cfg(test)]
    mod sbc {
        use super::*;
        use std::num::Wrapping;

        fn test(a: u8, b: u8, result: Flags) {
            let mut flags = Flags::empty();
            flags.zncv_sbc(a, b);
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
        fn flags_v() { test(0xfe, 0x7f, Flags::Overflow); }

        #[test]
        fn flags_ncv() { test(0x7f, 0xff, Flags::Negative | Flags::Carry | Flags::Overflow); }
    }
}


#[cfg(test)]
mod znc_cmp {
    use super::*;

    fn test(a: u8, b: u8, result: Flags) {
        let mut flags = Flags::empty();
        flags.znc_cmp(a, b);
        assert_eq!(flags, result, "\nexpr: 0x{:02x?} + 0x{:02x?}", a, b);
    }

    #[test]
    fn carry() { test(0xff, 0xfe, Flags::Carry); }

    #[test]
    fn zero_carry() { test(0xff, 0xff, Flags::Zero | Flags::Carry); }

    #[test]
    fn negative() { test(0x80, 0xff, Flags::Negative); }
}