use std::num::Wrapping;

// RAM Size
const RAM_CAPACITY: usize = 0x0800;

#[derive(Copy, Clone)]
pub struct Ram(pub [u8; RAM_CAPACITY]);

impl Ram {
    // Create a new CPU. It has the default "power up" state.
    pub fn new() -> Self { Ram([0; RAM_CAPACITY]) }

    // Read the value in RAM pointed by an address
    pub fn peek_at(&self, addr: u16) -> u8 { self.0[addr as usize] }

    // Read the value in RAM pointed by the least significant byte of an address
    pub fn peek_at_16(&self, addr: u16) -> u16 {
        let lsb = self.peek_at(addr) as u16;
        let msb = (self.peek_at((Wrapping(addr) + Wrapping(1)).0) as u16) << 8;
        msb + lsb
    }

    // Write the value to RAM pointed by the address
    pub fn put_at(&mut self, addr: u16, value: u8) {
        self.0[addr as usize] = value;
    }
}