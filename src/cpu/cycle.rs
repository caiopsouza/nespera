// This is a simplified cycle for an instruction on the 6502.
// Every instruction starts at T2 and finishes at T1 where the next opcode is fetched.
// For the actual cycle state see:
// - http://www.visual6502.org/wiki/index.php?title=6502_Timing_States
// - http://www.visual6502.org/wiki/index.php?title=6502_State_Machine
pub const T1: u8 = 1;
pub const T2: u8 = 2;
pub const T3: u8 = 3;
pub const T4: u8 = 4;
pub const T5: u8 = 5;
pub const T6: u8 = 6;
pub const T7: u8 = 7;
pub const T8: u8 = 8;

pub const FIRST: u8 = T2;
pub const LAST: u8 = T1;
pub const NEX_TO_LAST: u8 = LAST - 1;
