use crate::cartridge::location::Location;
use crate::cartridge::mapper::Mapper;

pub struct Mapper000;

impl Mapper000 { pub fn new() -> Self { Self } }

impl Mapper for Mapper000 {
    fn read_cpu(&self, addr: u16) -> Location {
        match addr {
            0x6000...0x7fff => Location::PrgRam(addr - 0x6000),
            0x8000...0xffff => Location::PrgRom(addr - 0x8000),
            _ => Location::Nowhere(addr),
        }
    }

    fn write_cpu(&self, addr: u16) -> Location {
        match addr {
            0x6000...0x7fff => Location::PrgRam(addr - 0x6000),
            _ => Location::Nowhere(addr),
        }
    }

    fn read_ppu(&self, addr: u16) -> Location {
        match addr {
            0x0000...0x1fff => Location::ChrRom(addr),
            0x2000...0x2fff => Location::PrgRam(addr - 0x2000),
            _ => Location::Nowhere(addr),
        }
    }

    fn write_ppu(&self, addr: u16) -> Location {
        match addr {
            0x0000...0x1fff => Location::ChrRom(addr),
            0x2000...0x2fff => Location::PrgRam(addr - 0x2000),
            _ => Location::Nowhere(addr),
        }
    }
}

impl Default for Mapper000 {
    fn default() -> Self { Self::new() }
}
