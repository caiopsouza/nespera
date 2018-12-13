use crate::cartridge::location::Location;

pub trait Mapper {
    fn read_cpu(&self, addr: u16) -> Location;
    fn write_cpu(&self, addr: u16) -> Location;

    fn read_ppu(&self, addr: u16) -> Location;
    fn write_ppu(&self, addr: u16) -> Location;
}
