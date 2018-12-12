pub use crate::mapper::cartridge::Cartridge;
use crate::mapper::location::Location;

pub mod cartridge;
pub mod location;
pub mod mapper000;

pub trait Mapper {
    fn read_cpu(&self, addr: u16) -> Location;
    fn write_cpu(&self, addr: u16) -> Location;

    fn read_ppu(&self, addr: u16) -> Location;
    fn write_ppu(&self, addr: u16) -> Location;
}
