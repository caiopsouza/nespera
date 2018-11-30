use std::fmt;

use pretty_hex::*;

// Memory capacity
const RAM_CAPACITY: usize = 0x0800;
const ROM_CAPACITY: usize = 0x4000;
const APU_CAPACITY: usize = 0x0018;

#[derive(Copy, Clone)]
pub struct Bus {
    pub ram: [u8; RAM_CAPACITY],
    pub rom: [u8; ROM_CAPACITY],
    pub apu: [u8; APU_CAPACITY],
}

impl Bus {
    pub fn new(mem: Vec<u8>) -> Self {
        let mut ram = [0; RAM_CAPACITY];
        let ram_len = RAM_CAPACITY.min(mem.len());
        ram[..ram_len].copy_from_slice(&mem[..ram_len]);

        let mut rom = [0; ROM_CAPACITY];
        let rom_len = ROM_CAPACITY.min(mem.len());
        rom[..rom_len].copy_from_slice(&mem[..rom_len]);

        Self { ram, rom, apu: [0; APU_CAPACITY] }
    }

    pub fn with_rom(rom: &[u8]) -> Self {
        let mut bus_rom = [0; ROM_CAPACITY];
        bus_rom.copy_from_slice(rom);

        Self { ram: [0; RAM_CAPACITY], rom: bus_rom, apu: [0; APU_CAPACITY] }
    }

    // Map an address to some data for reading and writing.
    fn map(&mut self, addr: u16) -> &mut u8 {
        unsafe {
            match addr {
                0x0000...0x1fff => self.ram.get_unchecked_mut(addr as usize % RAM_CAPACITY),
                0x8000...0xFFFF => self.rom.get_unchecked_mut((addr - 0x8000) as usize % ROM_CAPACITY),
                0x4000...0x4017 => self.apu.get_unchecked_mut((addr - 0x4000) as usize % ROM_CAPACITY),
                _ => panic!("Mapper not implemented for address 0x{:x}\n{:?}", addr, self)
            }
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 { *self.map(addr) }
    pub fn write(&mut self, addr: u16, data: u8) { *self.map(addr) = data }
}

impl fmt::Debug for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "RAM: {:?}", (&self.ram[..]).hex_dump())/*?;
        write!(formatter, "ROM: {:?}", (&self.rom[..]).hex_dump())*/
    }
}

impl PartialEq for Bus {
    fn eq(&self, other: &Bus) -> bool {
        self.ram.eq(&other.ram[..]) && self.rom.eq(&other.rom[..])
    }
}
