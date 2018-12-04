use std::ops::Range;

use crate::bus::Bus;

const EIGHT_KBYTES_IN_BYTES: usize = 8192;
const SIXTEEN_KBYTES_IN_BYTES: usize = 2 * EIGHT_KBYTES_IN_BYTES;
const PRG_ROM_START: usize = 0x10;

#[derive(Debug, Eq, PartialEq)]
pub enum InesLoadError {
    InvalidHeader,
    UnableToReadPrgRom,
    UnableToReadChrRom,
}

#[derive(Debug, Eq, PartialEq)]
pub struct INes {
    data: Vec<u8>,
    prg_rom: Range<usize>,
    chr_rom: Range<usize>,
}

impl INes {
    //impl INes {
    pub fn new(rom: Vec<u8>) -> Result<Self, InesLoadError> {
        // Check header
        if rom.get(0..4) != Some(&b"NES\x1a"[..]) { return Err(InesLoadError::InvalidHeader); }

        // PRG ROM has 0x04 * 16kb in size
        let prg_rom = PRG_ROM_START..PRG_ROM_START + rom[0x04] as usize * SIXTEEN_KBYTES_IN_BYTES;
        rom.get(prg_rom.clone()).ok_or(InesLoadError::UnableToReadPrgRom)?;

        // CHR ROM has 0x05 * 8kb in size
        let chr_rom_start_byte = prg_rom.end;
        let chr_rom = chr_rom_start_byte..chr_rom_start_byte + rom[0x05] as usize * EIGHT_KBYTES_IN_BYTES;
        rom.get(chr_rom.clone()).ok_or(InesLoadError::UnableToReadChrRom)?;

        Ok(Self { data: rom, prg_rom, chr_rom })
    }

    // Reads PRG ROM
    pub fn prg_rom(&self) -> &[u8] { &self.data[self.prg_rom.clone()] }

    // Reads CHR ROM
    pub fn chr_rom(&self) -> &[u8] { &self.data[self.chr_rom.clone()] }

    // Transform the file into a Bus
    pub fn into_bus(self) -> Bus {
        let mut bus = Bus::new();
        bus.cpu.rom.copy_from_slice(self.prg_rom());
        bus.ppu.ram[..0x2000].copy_from_slice(self.chr_rom());
        bus
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test<'a>(data: &[u8]) -> Vec<u8> { Vec::<u8>::from(data) }

    fn load_test() -> INes {
        let file = make_test(include_bytes!("../../tests/resources/nestest.nes"));
        INes::new(file).unwrap()
    }

    #[test]
    fn invalid_header() {
        assert_eq!(INes::new(make_test(b"666")), Result::Err(InesLoadError::InvalidHeader));
    }

    #[test]
    fn prg_rom_header() {
        assert_eq!(INes::new(make_test(b"NES\x1a666")), Result::Err(InesLoadError::UnableToReadPrgRom));
    }

    #[test]
    fn prg_rom_start() {
        let rom = load_test();
        assert_eq!(rom.prg_rom()[0], 0x4c);
    }

    #[test]
    fn prg_rom_end() {
        let rom = load_test();
        assert_eq!(rom.prg_rom()[0x3fff], 0xc5);
    }

    #[test]
    fn prg_chr_start() {
        let rom = load_test();
        assert_eq!(rom.chr_rom.start, 0x4010);
    }

    #[test]
    fn chr_rom_end() {
        let rom = load_test();
        assert_eq!(rom.chr_rom.end, 0x6010);
    }

    #[test]
    fn memory() {
        let rom = load_test();
        let mut bus = rom.into_bus();
        assert_eq!(bus.read(0xC000), 0x4C);
    }
}
