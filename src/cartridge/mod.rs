use std::cmp;
use std::fmt;
use std::fs::File;
use std::io;
use std::io::Read;

use pretty_hex::PrettyHex;

use crate::cartridge::location::Location;
use crate::cartridge::mapper000::Mapper000;
use crate::cartridge::mapper::Mapper;
use crate::utils::bits;

pub mod mapper;
pub mod location;
pub mod mapper000;

const EIGHT_KBYTES: usize = 0x2000;
const SIXTEEN_KBYTES: usize = 2 * EIGHT_KBYTES;
const PRG_ROM_START: usize = 0x10;

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    InvalidHeader,
    UnableToReadPrgRom,
    UnableToReadChrRom,
    MapperNotImplemented,
}

impl From<io::Error> for LoadError {
    fn from(error: io::Error) -> Self { LoadError::Io(error) }
}

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    mapper: Box<Mapper>,
}

impl Cartridge {
    pub fn new(data: &[u8]) -> Result<Self, LoadError> {
        // Check header
        if data.get(0..4) != Some(&b"NES\x1a"[..]) { return Err(LoadError::InvalidHeader); }

        // PRG ROM has 0x04 * 16kb in size
        let prg_rom = PRG_ROM_START..PRG_ROM_START + data[0x04] as usize * SIXTEEN_KBYTES;
        let prg_rom = data.get(prg_rom).ok_or(LoadError::UnableToReadPrgRom)?;
        let prg_rom = prg_rom.to_owned();

        // CHR ROM has 0x05 * 8kb in size
        let chr_rom_start_byte = PRG_ROM_START + prg_rom.len();
        let chr_rom = chr_rom_start_byte..chr_rom_start_byte + data[0x05] as usize * EIGHT_KBYTES;
        let chr_rom = data.get(chr_rom).ok_or(LoadError::UnableToReadChrRom)?;
        let chr_rom = chr_rom.to_owned();

        // Mapper.
        // High nybble of 6 contains the lower nybble of the mapper.
        // High nybble of 7 contains the higher nybble of the mapper.
        let mapper = ((data[0x06] & 0b1111_0000) >> 4) | (data[0x07] & 0b1111_0000);
        let mapper = match mapper {
            0 => box Mapper000::new(),
            _ => return Result::Err(LoadError::MapperNotImplemented),
        };

        // PRG RAM is present if bit is not set.
        let prg_ram_capacity =
            if bits::is_set(data[0x0a], 4) {
                0
            } else {
                EIGHT_KBYTES * cmp::max(data[0x08] as usize, 1)
            };

        Ok(
            Self {
                prg_rom,
                chr_rom,
                prg_ram: vec![0; prg_ram_capacity],
                mapper,
            }
        )
    }
    pub fn from_file(file: &str) -> Result<Self, LoadError> {
        let mut file = File::open(file)?;
        let mut data = Vec::<u8>::new();
        file.read_to_end(&mut data)?;
        Self::new(&data)
    }

    pub fn empty() -> Self {
        Self {
            prg_rom: vec![0; SIXTEEN_KBYTES],
            chr_rom: vec![0; EIGHT_KBYTES],
            prg_ram: vec![0; 0],
            mapper: box Mapper000::new(),
        }
    }

    pub fn read_prg_rom(&self, addr: u16) -> u8 {
        let index = addr as usize % self.prg_rom.len();
        unsafe { *self.prg_rom.get_unchecked(index) }
    }

    pub fn read_chr_rom(&self, addr: u16) -> u8 {
        let index = addr as usize % self.chr_rom.len();
        unsafe { *self.chr_rom.get_unchecked(index) }
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

impl fmt::Debug for Cartridge {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "PRG ROM | {:?}\n", &self.prg_rom.hex_dump())?;
        writeln!(formatter, "CHR ROM | {:?}\n", &self.chr_rom.hex_dump())?;
        write!(formatter, "PRG RAM | {:?}", (&self.prg_ram[..]).hex_dump())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_test() -> Cartridge { Cartridge::from_file("tests/resources/cpu/nestest.nes").unwrap() }

    #[test]
    fn invalid_header() {
        let cartridge = Cartridge::new(&b"666"[..]);
        assert!(match cartridge.expect_err("") {
            LoadError::InvalidHeader => true,
            _ => false
        });
    }

    #[test]
    fn prg_rom_header() {
        let cartridge = Cartridge::new(&b"NES\x1a666"[..]);
        assert!(match cartridge.expect_err("") {
            LoadError::UnableToReadPrgRom => true,
            _ => false
        });
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
}
