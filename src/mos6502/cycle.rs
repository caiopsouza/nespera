// This is a simplified cycle for an instruction on the 6502.
// Every instruction starts at T2 and finishes at T1 where the next opcode is fetched.
// For the actual cycle state see:
// - http://www.visual6502.org/wiki/index.php?title=6502_Timing_States
// - http://www.visual6502.org/wiki/index.php?title=6502_State_Machine
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Cycle(pub u8);

pub const T1: Cycle = Cycle(1);
pub const T2: Cycle = Cycle(2);
pub const T3: Cycle = Cycle(3);
pub const T4: Cycle = Cycle(4);
pub const T5: Cycle = Cycle(5);
pub const T6: Cycle = Cycle(6);
pub const T7: Cycle = Cycle(7);
pub const T8: Cycle = Cycle(8);

pub const FIRST: Cycle = T2;
pub const LAST: Cycle = T1;
