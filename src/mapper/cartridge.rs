use std::fmt;
use std::ops::Range;

use pretty_hex::PrettyHex;

use crate::mapper::Location;
use crate::mapper::Mapper;
use crate::mapper::mapper000::Mapper000;

const EIGHT_KBYTES: usize = 0x2000;
const SIXTEEN_KBYTES: usize = 2 * EIGHT_KBYTES;
const PRG_ROM_START: usize = 0x10;

#[derive(Debug, Eq, PartialEq)]
pub enum CartridgeLoadError {
    InvalidHeader,
    UnableToReadPrgRom,
    UnableToReadChrRom,
}

pub struct Cartridge {
    file: Vec<u8>,
    prg_rom: Range<usize>,
    chr_rom: Range<usize>,
    pub prg_ram: Vec<u8>,
    mapper: Box<Mapper>,
}

impl Cartridge {
    pub fn new(file: Vec<u8>) -> Result<Self, CartridgeLoadError> {
        // Check header
        if file.get(0..4) != Some(&b"NES\x1a"[..]) { return Err(CartridgeLoadError::InvalidHeader); }

        // PRG ROM has 0x04 * 16kb in size
        let prg_rom = PRG_ROM_START..PRG_ROM_START + file[0x04] as usize * SIXTEEN_KBYTES;
        file.get(prg_rom.clone()).ok_or(CartridgeLoadError::UnableToReadPrgRom)?;

        // CHR ROM has 0x05 * 8kb in size
        let chr_rom_start_byte = prg_rom.end;
        let chr_rom = chr_rom_start_byte..chr_rom_start_byte + file[0x05] as usize * EIGHT_KBYTES;
        file.get(chr_rom.clone()).ok_or(CartridgeLoadError::UnableToReadChrRom)?;

        Ok(
            Self {
                file,
                prg_rom,
                chr_rom,
                prg_ram: vec![0; EIGHT_KBYTES],
                mapper: box Mapper000::new(),
            }
        )
    }

    pub fn empty() -> Self {
        Self {
            file: vec![0],
            prg_rom: 0..1,
            chr_rom: 0..1,
            prg_ram: vec![0; EIGHT_KBYTES],
            mapper: box Mapper000::new(),
        }
    }

    pub fn read_chr_rom(&self, addr: usize) -> u8 {
        unsafe {
            let chr_rom = self.file.get_unchecked(self.chr_rom.clone());
            *chr_rom.get_unchecked(addr % chr_rom.len())
        }
    }

    fn read(&self, location: Location) -> u8 {
        unsafe {
            match location {
                Location::Nowhere(addr) => {
                    error!("Mapper attempted to read from nowhere in CPU at address {:#04x}. Defaulting to zero.", addr);
                    0
                }

                Location::PrgRam(addr) => {
                    *self.prg_ram.get_unchecked(addr as usize % self.prg_ram.len())
                }

                Location::PrgRom(addr) => {
                    let prg_rom = self.file.get_unchecked(self.prg_rom.clone());
                    *prg_rom.get_unchecked(addr as usize % self.prg_rom.len())
                }

                Location::ChrRom(addr) => {
                    let chr_rom = self.file.get_unchecked(self.chr_rom.clone());
                    *chr_rom.get_unchecked(addr as usize % self.chr_rom.len())
                }
            }
        }
    }

    pub fn write(&mut self, location: Location, data: u8) {
        unsafe {
            match location {
                Location::Nowhere(addr) => {
                    error!("Mapper attempted to write into nowhere in CPU at address {:#04x}.", addr);
                }

                Location::PrgRam(addr) => {
                    let prg_ram_len = self.prg_ram.len();
                    *self.prg_ram.get_unchecked_mut(addr as usize % prg_ram_len) = data
                }

                Location::PrgRom(_) | Location::ChrRom(_) => {
                    error!("Mapper attempted to write into read only memory in CPU at location {:#04x?}", location)
                }
            }
        }
    }

    pub fn read_cpu(&self, addr: u16) -> u8 {
        let location = self.mapper.read_cpu(addr);
        self.read(location)
    }

    pub fn write_cpu(&mut self, addr: u16, data: u8) {
        let location = self.mapper.write_cpu(addr);
        self.write(location, data)
    }

    pub fn read_ppu(&self, addr: u16) -> u8 {
        let location = self.mapper.read_ppu(addr);
        self.read(location)
    }

    pub fn write_ppu(&mut self, addr: u16, data: u8) {
        let location = self.mapper.write_ppu(addr);
        self.write(location, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test() -> Cartridge {
        let file = include_bytes!("../../tests/resources/cpu/nestest.nes")[..].to_owned();
        Cartridge::new(file).unwrap()
    }

    #[test]
    fn invalid_header() {
        let cartridge = Cartridge::new(b"666"[..].to_owned());
        assert_eq!(cartridge.err(), Option::Some(CartridgeLoadError::InvalidHeader));
    }

    #[test]
    fn prg_rom_header() {
        let cartridge = Cartridge::new(b"NES\x1a666"[..].to_owned());
        assert_eq!(cartridge.err(), Option::Some(CartridgeLoadError::UnableToReadPrgRom));
    }

    #[test]
    fn prg_rom_start() {
        let rom = load_test();
        assert_eq!(rom.read_cpu(0x8000), 0x4c);
    }

    #[test]
    fn prg_rom_end() {
        let rom = load_test();
        assert_eq!(rom.read_cpu(0x8000 + 0x3fff), 0xc5);
    }

    #[test]
    fn prg_chr_start() {
        let rom = load_test();
        assert_eq!(rom.read_cpu(0x8000 + 0x3fff), 0xc5);
        assert_eq!(rom.chr_rom.start, 0x4010);
    }

    #[test]
    fn chr_rom_end() {
        let rom = load_test();
        assert_eq!(rom.chr_rom.end, 0x6010);
    }
}

impl fmt::Debug for Cartridge {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "PRG ROM | {:?}", (&self.file[self.prg_rom.clone()]).hex_dump())?;
        writeln!(formatter, "CHR ROM | {:?}", (&self.file[self.chr_rom.clone()]).hex_dump())?;
        writeln!(formatter, "PRG RAM | {:?}", (&self.prg_ram[..]).hex_dump())
    }
}
