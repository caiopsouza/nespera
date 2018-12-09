use std::fmt;

use pretty_hex::*;

// Memory capacity
pub const RAM_CAPACITY: usize = 0x0800;

pub struct CpuData {
    ram: [u8; RAM_CAPACITY],
}

impl CpuData {
    pub fn new() -> Self { Self { ram: [0; RAM_CAPACITY] } }

    pub fn with_mem(mem: Vec<u8>) -> Self {
        let mut res = Self::new();

        let ram_len = RAM_CAPACITY.min(mem.len());
        res.ram[..ram_len].copy_from_slice(&mem[..ram_len]);

        res
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        unsafe { *self.ram.get_unchecked((addr - 0x0000) as usize % RAM_CAPACITY) }
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        unsafe { *self.ram.get_unchecked_mut((addr - 0x0000) as usize % RAM_CAPACITY) = data }
    }
}

impl fmt::Debug for CpuData {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "RAM | {:?}", (&self.ram[..]).hex_dump())
    }
}
