use std::fmt;

use pretty_hex::*;

// Memory capacity
const RAM_CAPACITY: usize = 0x0800;
const ROM_CAPACITY: usize = 0x4000;
const APU_CAPACITY: usize = 0x0018;
const PPU_CAPACITY: usize = 0x0008;

#[derive(Clone)]
pub struct Bus {
    pub ram: [u8; RAM_CAPACITY],
    pub rom: [u8; ROM_CAPACITY],
    pub apu: [u8; APU_CAPACITY],
    pub ppu: [u8; PPU_CAPACITY],

    // Dummy byte used to read and write areas not mapped.
    pub dummy: u8,
}

impl Bus {
    pub fn new() -> Self {
        let mut ppu = [0; PPU_CAPACITY];
        ppu[2] = 0b10000000; // V blank

        Self {
            ram: [0; RAM_CAPACITY],
            ppu,
            rom: [0; ROM_CAPACITY],
            apu: [0; APU_CAPACITY],
            dummy: 0,
        }
    }

    pub fn with_mem(mem: Vec<u8>) -> Self {
        let mut res = Self::new();

        let ram_len = RAM_CAPACITY.min(mem.len());
        res.ram[..ram_len].copy_from_slice(&mem[..ram_len]);

        let rom_len = ROM_CAPACITY.min(mem.len());
        res.rom[..rom_len].copy_from_slice(&mem[..rom_len]);

        res
    }

    // Map an address to some data for reading and writing.
    fn map(&mut self, addr: u16) -> &mut u8 {
        unsafe {
            match addr {
                0x0000...0x1fff => self.ram.get_unchecked_mut(addr as usize % RAM_CAPACITY),
                0x2000...0x3FFF => self.ppu.get_unchecked_mut((addr - 0x2000) as usize % PPU_CAPACITY),
                0x8000...0xFFFF => self.rom.get_unchecked_mut((addr - 0x8000) as usize % ROM_CAPACITY),
                0x4000...0x4017 => self.apu.get_unchecked_mut((addr - 0x4000) as usize % ROM_CAPACITY),
                _ => {
                    eprintln!("Warning: access to bus area not mapped.");
                    self.dummy = 0;
                    &mut self.dummy
                }
            }
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 { *self.map(addr) }
    pub fn write(&mut self, addr: u16, data: u8) { *self.map(addr) = data }
}

impl fmt::Debug for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "Ram | {:?}", (&self.ram[..]).hex_dump())?;
        writeln!(formatter, "Ppu | {:?}", (&self.ppu[..]).hex_dump())?;
        writeln!(formatter, "Rom | {:?}", (&self.rom[..]).hex_dump())?;
        write!(formatter, "Apu | {:?}", (&self.apu[..]).hex_dump())
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl PartialEq for Bus {
    fn eq(&self, other: &Bus) -> bool {
        self.ram.eq(&other.ram[..])
            && self.rom.eq(&other.rom[..])
            && self.apu.eq(&other.apu[..])
    }
}
