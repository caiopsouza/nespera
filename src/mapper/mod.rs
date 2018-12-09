pub use crate::mapper::cartridge::Cartridge;

pub mod cartridge;
pub mod mapper000;

pub enum Location {
    Nowhere,
    PrgRam(u16),
    PrgRom(u16),
    ChrRom(u16),
}

pub trait Mapper {
    fn read(&self, addr: u16) -> Location;
    fn write(&self, addr: u16) -> Location;
}
