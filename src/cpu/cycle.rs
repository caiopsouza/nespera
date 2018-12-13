// This is a simplified cycle for an instruction on the 6502.
// Every instruction starts at T2 and finishes at T1 where the next opcode is fetched.
// For the actual cycle state see:
// - http://www.visual6502.org/wiki/index.php?title=6502_Timing_States
// - http://www.visual6502.org/wiki/index.php?title=6502_State_Machine
pub const T1: u16 = 1;
pub const T2: u16 = 2;
pub const T3: u16 = 3;
pub const T4: u16 = 4;
pub const T5: u16 = 5;
pub const T6: u16 = 6;
pub const T7: u16 = 7;
pub const T8: u16 = 8;
pub const T9: u16 = 9;
pub const T10: u16 = 10;

pub const FIRST: u16 = T2;
pub const LAST: u16 = T1;
pub const NEX_TO_LAST: u16 = LAST - 1;
