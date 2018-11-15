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
}

impl From<u8> for Flags {
    fn from(bits: u8) -> Self { Self { bits } }
}

#[cfg(test)]
mod zn {
    use super::*;

    #[test]
    fn zero() {
        let mut flags = Flags::empty();
        flags.zn(0);
        assert_eq!(flags.bits(), 0b00000010);
    }

    #[test]
    fn positive() {
        let mut flags = Flags::all();
        flags.zn(127);
        assert_eq!(flags.bits(), 0b01111101);
    }

    #[test]
    fn negative() {
        let mut flags = Flags::empty();
        flags.zn(128);
        assert_eq!(flags.bits(), 0b10000000);
    }
}

#[cfg(test)]
mod zno_bit_test {
    use super::*;

    #[test]
    fn zero() {
        let mut flags = Flags::empty();
        flags.zno_bit_test(0);
        assert_eq!(flags.bits(), 0b00000010);
    }

    #[test]
    fn positive() {
        let mut flags = Flags::all();
        flags.zno_bit_test(63);
        assert_eq!(flags.bits(), 0b00111101);
    }

    #[test]
    fn positive_overflow() {
        let mut flags = Flags::all();
        flags.zno_bit_test(127);
        assert_eq!(flags.bits(), 0b01111101);
    }

    #[test]
    fn negative() {
        let mut flags = Flags::empty();
        flags.zno_bit_test(128);
        assert_eq!(flags.bits(), 0b10000000);
    }

    #[test]
    fn negative_overflow() {
        let mut flags = Flags::empty();
        flags.zno_bit_test(204);
        assert_eq!(flags.bits(), 0b11000000);
    }
}