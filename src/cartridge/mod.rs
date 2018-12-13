use pretty_hex::PrettyHex;
use std::cmp;
use std::fmt;
use std::ops::Range;

use crate::utils::bits;
use crate::cartridge::mapper::Mapper;
use crate::cartridge::mapper000::Mapper000;
use crate::cartridge::location::Location;

pub mod mapper;
pub mod location;
pub mod mapper000;

const EIGHT_KBYTES: usize = 0x2000;
const SIXTEEN_KBYTES: usize = 2 * EIGHT_KBYTES;
const PRG_ROM_START: usize = 0x10;

#[derive(Debug, Eq, PartialEq)]
pub enum LoadError {
    InvalidHeader,
    UnableToReadPrgRom,
    UnableToReadChrRom,
    MapperNotImplemented,
}

pub struct Cartridge {
    file: Vec<u8>,
    prg_rom: Range<usize>,
    chr_rom: Range<usize>,
    prg_ram: Vec<u8>,
    mapper: Box<Mapper>,
}

impl Cartridge {
    pub fn new(file: Vec<u8>) -> Result<Self, LoadError> {
        // Check header
        if file.get(0..4) != Some(&b"NES\x1a"[..]) { return Err(LoadError::InvalidHeader); }

        // PRG ROM has 0x04 * 16kb in size
        let prg_rom = PRG_ROM_START..PRG_ROM_START + file[0x04] as usize * SIXTEEN_KBYTES;
        file.get(prg_rom.clone()).ok_or(LoadError::UnableToReadPrgRom)?;

        // CHR ROM has 0x05 * 8kb in size
        let chr_rom_start_byte = prg_rom.end;
        let chr_rom = chr_rom_start_byte..chr_rom_start_byte + file[0x05] as usize * EIGHT_KBYTES;
        file.get(chr_rom.clone()).ok_or(LoadError::UnableToReadChrRom)?;

        // Mapper.
        // High nybble of 6 contains the lower nybble of the mapper.
        // High nybble of 7 contains the higher nybble of the mapper.
        let mapper = ((file[0x06] & 0b1111_0000) >> 4) | (file[0x07] & 0b1111_0000);
        let mapper = match mapper {
            0 => box Mapper000::new(),
            _ => return Result::Err(LoadError::MapperNotImplemented),
        };

        // PRG RAM is present if bit is not set.
        let prg_ram_capacity =
            if bits::is_set(file[0x0a], 4) {
                0
            } else {
                EIGHT_KBYTES * cmp::max(file[0x08] as usize, 1)
            };

        Ok(
            Self {
                file,
                prg_rom,
                chr_rom,
                prg_ram: vec![0; prg_ram_capacity],
                mapper,
            }
        )
    }

    pub fn empty() -> Self {
        Self {
            file: vec![0],
            prg_rom: 0..1,
            chr_rom: 0..1,
            prg_ram: vec![0; 0],
            mapper: box Mapper000::new(),
        }
    }

    pub fn read_prg_rom(&self, addr: u16) -> u8 {
        unsafe {
            let prg_rom = self.file.get_unchecked(self.prg_rom.clone());
            *prg_rom.get_unchecked(addr as usize % prg_rom.len())
        }
    }

    pub fn read_chr_rom(&self, addr: u16) -> u8 {
        unsafe {
            let chr_rom = self.file.get_unchecked(self.chr_rom.clone());
            *chr_rom.get_unchecked(addr as usize % chr_rom.len())
        }
    }

    pub fn read_prg_ram(&self, addr: u16) -> u8 {
        if self.prg_ram.is_empty() {
            error!("Attempt to read from PRG RAM, but cartridge reports it's not present. Defaulting to zero. 0x{:04x}",
                   addr);
            return 0;
        }

        let index = addr as usize % self.prg_ram.len();
        unsafe { *self.prg_ram.get_unchecked(index) }
    }

    pub fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if self.prg_ram.is_empty() {
            error!("Attempt to write to PRG RAM, but cartridge reports it's not present. Defaulting to zero. 0x{:04x}, 0x{:02x}",
                   addr, data);
            return;
        }

        let index = addr as usize % self.prg_ram.len();
        unsafe { *self.prg_ram.get_unchecked_mut(index) = data }
    }

    // Canonize a PPU register from the mirrored area
    fn canon_ppu_register_addr(addr: u16) -> u16 {
        const PPU_REG_AMOUNT: u16 = 0x08;
        const PPU_REGS_ADDR_START: u16 = 0x2000;
        (addr - PPU_REGS_ADDR_START) % PPU_REG_AMOUNT + PPU_REGS_ADDR_START
    }

    // Common PRG RAM location
    fn prg_ram_location(&self, addr: u16) -> Location {
        match self.prg_ram.len() {
            0 => Location::Nowhere(addr),
            _ => Location::PrgRam(addr - 0x6000),
        }
    }

    // Common cpu locations
    fn cpu_location(&self, addr: u16) -> Location {
        #[allow(clippy::match_overlapping_arm)] match addr {
            0x0000...0x1fff => Location::CpuRam(addr),

            0x2000...0x3fff => {
                let addr = Self::canon_ppu_register_addr(addr);
                match addr {
                    0x2000 => Location::PpuCtrl,
                    0x2001 => Location::PpuMask,
                    0x2002 => Location::PpuStatus,
                    0x2003 => Location::OamAddr,
                    0x2004 => Location::OamData,
                    0x2005 => Location::PpuScroll,
                    0x2006 => Location::PpuAddr,
                    0x2007 => Location::PpuData,
                    _ => unimplemented!("Canonical PPU register doesn't exist for address 0x{:04x}.", addr),
                }
            }

            0x4014 => Location::OamDma,

            0x4000...0x4017 => {
                warn!("Reading from APU address 0x{:04x}.", addr);
                Location::Apu(addr - 0x4000)
            }

            _ => {
                error!("Reading from area not mapped in CPU. Addr 0x{:04x}.", addr);
                Location::Nowhere(addr)
            }
        }
    }

    // Location for reading from the CPU
    pub fn cpu_read_location(&self, addr: u16) -> Location {
        match addr {
            0x6000...0x7fff => self.prg_ram_location(addr),
            0x8000...0xffff => self.mapper.read_cpu(addr),
            _ => self.cpu_location(addr)
        }
    }

    // Location for writing from the CPU
    pub fn cpu_write_location(&self, addr: u16) -> Location {
        match addr {
            0x6000...0x7fff => self.prg_ram_location(addr),
            0x8000...0xffff => self.mapper.write_cpu(addr),
            _ => self.cpu_location(addr)
        }
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
        assert_eq!(cartridge.err(), Option::Some(LoadError::InvalidHeader));
    }

    #[test]
    fn prg_rom_header() {
        let cartridge = Cartridge::new(b"NES\x1a666"[..].to_owned());
        assert_eq!(cartridge.err(), Option::Some(LoadError::UnableToReadPrgRom));
    }

    #[test]
    fn prg_rom_start() {
        let cartridge = load_test();
        assert_eq!(cartridge.cpu_read_location(0x8000), Location::PrgRom(0x0000));
    }

    #[test]
    fn prg_rom_end() {
        let cartridge = load_test();
        assert_eq!(cartridge.cpu_read_location(0x8000 + 0x3fff), Location::PrgRom(0x3fff));
    }

    #[test]
    fn prg_chr_start() {
        let cartridge = load_test();
        assert_eq!(cartridge.chr_rom.start, 0x4010);
    }

    #[test]
    fn chr_rom_end() {
        let cartridge = load_test();
        assert_eq!(cartridge.chr_rom.end, 0x6010);
    }
}

impl fmt::Debug for Cartridge {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "PRG ROM | {:?}\n", (&self.file[self.prg_rom.clone()]).hex_dump())?;
        writeln!(formatter, "CHR ROM | {:?}\n", (&self.file[self.chr_rom.clone()]).hex_dump())?;
        write!(formatter, "PRG RAM | {:?}", (&self.prg_ram[..]).hex_dump())
    }
}
