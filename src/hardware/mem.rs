use std::num::Wrapping;
use std::fmt;

use pretty_hex::*;

// RAM Size
const RAM_CAPACITY: usize = 0x0800;
const ROM_CAPACITY: usize = 0x4000;

#[derive(Copy, Clone)]
pub struct Memory {
    ram: [u8; RAM_CAPACITY],
    rom: [u8; ROM_CAPACITY],
}

impl Memory {
    // Create a new CPU. It has the default "power up" state.
    pub fn new() -> Self {
        Self {
            ram: [0; RAM_CAPACITY],
            rom: [0; ROM_CAPACITY],
        }
    }

    pub fn new_from_rom(rom: &[u8]) -> Self {
        let mut mem_rom = [0; ROM_CAPACITY];
        mem_rom.copy_from_slice(rom);

        Self {
            ram: [0; RAM_CAPACITY],
            rom: mem_rom,
        }
    }

    // Returns a reference to RAM
    pub fn get_ram(&self) -> &[u8; RAM_CAPACITY] { &self.ram }

    // Map an address
    fn map(&self, addr: u16) -> &u8 {
        unsafe {
            match addr {
                0x0000...0x1fff => self.ram.get_unchecked(addr as usize % RAM_CAPACITY),
                0x8000...0xFFFF => self.rom.get_unchecked(addr as usize % ROM_CAPACITY),
                _ => panic!("Mapper not implemented for address 0x{:x}", addr)
            }
        }
    }

    // Map an address to a mutable reference
    fn map_as_mut(&mut self, addr: u16) -> &mut u8 {
        unsafe {
            match addr {
                0x0000...0x1fff => self.ram.get_unchecked_mut(addr as usize % RAM_CAPACITY),
                0x8000...0xFFFF => self.rom.get_unchecked_mut(addr as usize % ROM_CAPACITY),
                _ => panic!("Mapper not implemented for address 0x{:x}", addr)
            }
        }
    }

    // Read the value in RAM pointed by an address
    pub fn peek_at(&self, addr: u16) -> u8 { *self.map(addr) }

    // Read the value in RAM pointed by the least significant byte of an address
    pub fn peek_at_16(&self, addr: u16) -> u16 {
        let lsb = self.peek_at(addr) as u16;
        let msb = (self.peek_at((Wrapping(addr) + Wrapping(1)).0) as u16) << 8;
        msb + lsb
    }

    // Write the value to RAM pointed by the address
    pub fn put_at(&mut self, addr: u16, value: u8) { *self.map_as_mut(addr) = value; }

    // Write the value to RAM pointed by the address
    pub fn put_at_16(&mut self, addr: u16, value: u16) {
        self.put_at(addr, value as u8);
        self.put_at((Wrapping(addr) + Wrapping(1)).0, (value >> 8) as u8);
    }
}

impl fmt::Debug for Memory {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "RAM: {:?}", (&self.ram[..]).hex_dump())?;
        write!(formatter, "ROM: {:?}", (&self.rom[..]).hex_dump())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ram_read() {
        let mut mem = Memory::new();
        mem.put_at(0x10u16, 0xa9u8);
        for i in 0..4 {
            assert_eq!(mem.peek_at(0x0800u16 * i + 0x10u16), 0xa9u8);
        }
    }

    #[test]
    fn ram_write() {
        let mut mem = Memory::new();
        mem.put_at(0x10u16 + 0x0800u16 * 3, 0xa9u8);
        for i in 0..4 {
            assert_eq!(mem.peek_at(0x0800u16 * i + 0x10u16), 0xa9u8);
        }
    }
}