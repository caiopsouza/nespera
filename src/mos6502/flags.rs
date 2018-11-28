// Bit for each flag
pub const CARRY: u8 = 0b00000001;
pub const ZERO: u8 = 0b00000010;
pub const INTERRUPT_DISABLE: u8 = 0b00000100;
pub const DECIMAL_MODE: u8 = 0b00001000;
pub const BREAK_COMMAND: u8 = 0b00010000;
pub const UNUSED: u8 = 0b00100000;
pub const OVERFLOW: u8 = 0b01000000;
pub const NEGATIVE: u8 = 0b10000000;

pub const LEAST_BIT: u8 = 0b10000000;

// Flags for the P register
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Flags(pub u8);

impl From<u8> for Flags {
    fn from(data: u8) -> Self { Self(data) }
}

impl Flags {
    pub fn change(&self, other: Flags, condition: bool) -> Flags {
        let res = if condition { self.0 | other.0 } else { self.0 & !other.0 };
        res.into()
    }
}