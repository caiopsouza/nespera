pub use crate::mapper::cartridge::Cartridge;

pub mod cartridge;
pub mod mapper000;

#[derive(Debug)]
pub enum Location {
    Nowhere(u16),
    PrgRam(u16),
    PrgRom(u16),
    ChrRom(u16),
}

pub trait Mapper {
    fn read_cpu(&self, addr: u16) -> Location;
    fn write_cpu(&self, addr: u16) -> Location;

    fn read_ppu(&self, addr: u16) -> Location;
    fn write_ppu(&self, addr: u16) -> Location;
}
