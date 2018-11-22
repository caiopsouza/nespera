use std::fmt;

use pretty_hex::*;
use crate::hardware::bus::Bus;

// Memory capacity
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

    // Create a new CPU. It has the default "power up" state.
    pub fn with_rom(rom: &[u8]) -> Self {
        let mut prg_rom = [0; ROM_CAPACITY];
        prg_rom.copy_from_slice(rom);

        Self {
            ram: [0; RAM_CAPACITY],
            rom: prg_rom,
        }
    }

    fn map(&mut self, addr: u16) -> &mut u8 {
        unsafe {
            match addr {
                0x0000...0x1fff => self.ram.get_unchecked_mut(addr as usize % RAM_CAPACITY),
                0x8000...0xFFFF => self.rom.get_unchecked_mut((addr - 0x8000) as usize % ROM_CAPACITY),
                _ => panic!("Mapper not implemented for address 0x{:x}\n{:?}", addr, self)
            }
        }
    }
}

impl Bus for Memory {
    fn read(&mut self, addr: u16) -> u8 { *self.map(addr) }
    fn write(&mut self, addr: u16, data: u8) { *self.map(addr) = data; }
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
        mem.write(0x10u16, 0xa9u8);
        for i in 0..4 {
            assert_eq!(mem.read(0x0800u16 * i + 0x10u16), 0xa9u8);
        }
    }

    #[test]
    fn ram_write() {
        let mut mem = Memory::new();
        mem.write(0x10u16 + 0x0800u16 * 3, 0xa9u8);
        for i in 0..4 {
            assert_eq!(mem.read(0x0800u16 * i + 0x10u16), 0xa9u8);
        }
    }
}