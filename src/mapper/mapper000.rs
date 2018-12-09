use crate::mapper::Location;
use crate::mapper::Mapper;

pub struct Mapper000;

impl Mapper000 { pub fn new() -> impl Mapper { Self } }

impl Mapper for Mapper000 {
    fn read(&self, addr: u16) -> Location {
        match addr {
            0x6000...0x7fff => Location::PrgRam(addr - 0x6000),
            0x8000...0xffff => Location::PrgRom(addr - 0x8000),
            _ => {
                error!("Attempt to read address {:#04x} from Mapper 000.", addr);
                Location::Nowhere
            }
        }
    }

    fn write(&self, addr: u16) -> Location {
        match addr {
            0x6000...0x7fff => Location::PrgRam(addr - 0x6000),
            0x8000...0xffff => Location::PrgRom(addr - 0x8000),
            _ => {
                error!("Attempt to write to address {:#04x} from Mapper 000.", addr);
                Location::Nowhere
            }
        }
    }
}
