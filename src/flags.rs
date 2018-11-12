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
}

impl From<u8> for Flags {
    fn from(bits: u8) -> Self { Self { bits } }
}
