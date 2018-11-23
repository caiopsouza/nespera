// Flags for the P register
pub const CARRY: u8 = 0b00000001;
pub const ZERO: u8 = 0b00000010;
pub const INTERRUPT_DISABLE: u8 = 0b00000100;
pub const DECIMAL_MODE: u8 = 0b00001000;
pub const BREAK_COMMAND: u8 = 0b00010000;
pub const UNUSED: u8 = 0b00100000;
pub const OVERFLOW: u8 = 0b01000000;
pub const NEGATIVE: u8 = 0b10000000;

// Change the value of a flag
pub fn change(flag: u8, flags_to_change: u8, condition: bool) -> u8 {
    if condition { flag | flags_to_change } else { flag & !flags_to_change }
}

pub fn change_zero(flag: u8, value: u8) -> u8 {
    change(flag, ZERO, value == 0)
}

pub fn change_negative(flag: u8, value: u8) -> u8 {
    // Flag Negative is the 7th bit and should be set when the 7th bit of the value is set.
    // Just copy it, then.
    (flag & !NEGATIVE) | (value & NEGATIVE)
}

pub fn change_zero_negative(flag: u8, value: u8) -> u8 {
    let res = change_zero(flag, value);
    change_negative(res, value)
}