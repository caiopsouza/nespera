#[derive(Debug, PartialOrd, PartialEq, Copy, Clone)]
pub struct Cpu {
    // Accumulator
    pub a: u8,

    // Index X
    pub x: u8,

    // Index Y
    pub y: u8,

    // Flags
    pub p: u8,

    // Program counter
    pub pc: u16,

    // Stack pointer
    pub s: u8,
}

impl Cpu {
    // Create a new CPU. It has the default "power up" state.
    pub fn new() -> Self { Self { a: 0, x: 0, y: 0, pc: 0, s: 0, p: 0 } }

    // Increment PC
    pub fn inc_pc(&self) { self.pc.wrapping_add(1); }
}
