#![allow(unused_variables)]

use std::fmt;

use pretty_hex::PrettyHex;

use crate::bus::cpu_data::CpuData;
use crate::bus::ppu_data::PpuData;
use crate::mapper::Cartridge;
use crate::mapper::location::Location;

mod cpu_data;
mod ppu_data;

const APU_CAPACITY: usize = 0x0018;

// General communication between all parts of the NES
pub struct Bus {
    // Interrupts
    pub reset: bool,
    pub nmi: bool,
    pub irq: bool,

    // Data
    pub cpu: CpuData,
    pub ppu: PpuData,
    pub apu: [u8; APU_CAPACITY],

    // Cartridge
    pub cartridge: Cartridge,
}

impl Bus {
    fn create(cpu: CpuData, cartridge: Cartridge) -> Self {
        Self {
            reset: true,
            nmi: false,
            irq: false,

            cpu,
            ppu: PpuData::new(),

            apu: [0; APU_CAPACITY],

            cartridge,
        }
    }

    pub fn new() -> Self {
        Self::create(CpuData::new(), Cartridge::empty())
    }

    pub fn with_mem(mem: Vec<u8>) -> Self {
        Self::create(CpuData::with_ram(mem), Cartridge::empty())
    }

    pub fn with_cartridge(cartridge: Cartridge) -> Self {
        Self::create(CpuData::new(), cartridge)
    }

    // Vblank has started
    pub fn start_vblank(&mut self) {
        self.ppu.vblank_set();
        if self.ppu.generate_nmi_at_vblank { self.nmi = true }
    }

    // Trace reading operations
    fn trace_read(location: &str, data: u8) -> u8 {
        trace!("Reading from {}: 0x{:02x}", location, data);
        data
    }
    fn trace_addr_read(location: &str, addr: u16, data: u8) -> u8 {
        trace!("Reading from {}: 0x{:04x}, 0x{:02x}", location, addr, data);
        data
    }

    // Read a value from the location
    fn read(&mut self, location: Location) -> u8 {
        match location {
            Location::Nowhere(addr) => {
                error!("Attempted to read from nowhere in CPU. Defaulting to zero. 0x{:02x}", addr);
                0
            }

            Location::Apu(addr) => {
                let data = unsafe { *self.apu.get_unchecked(addr as usize % APU_CAPACITY) };
                Self::trace_addr_read("APU", addr, data)
            }

            Location::CpuRam(addr) => {
                Self::trace_addr_read("CPU RAM", addr, self.cpu.read_ram(addr))
            }

            Location::PpuData => {
                Self::trace_read("PPUDATA", self.ppu.read_data())
            }

            Location::PpuStatus => {
                Self::trace_read("PPUSTATUS", self.ppu.read_status())
            }

            Location::OamData => {
                Self::trace_read("OAMDATA", self.ppu.read_oam_data())
            }

            Location::OamDma => { unimplemented!() }

            Location::PpuCtrl
            | Location::PpuMask
            | Location::OamAddr
            | Location::PpuAddr
            | Location::PpuScroll => {
                warn!("Reading from write only PPU area: 0x{:04x?}, 0x{:02x}.", location, self.ppu.latch);
                self.ppu.latch
            }

            Location::PrgRam(addr) => {
                Self::trace_addr_read("PRG RAM", addr, self.cartridge.read_prg_ram(addr))
            }

            Location::PrgRom(addr) => {
                Self::trace_addr_read("PRG ROM", addr, self.cartridge.read_prg_rom(addr))
            }

            Location::ChrRom(addr) => {
                Self::trace_addr_read("CHR ROM", addr, self.cartridge.read_chr_rom(addr))
            }
        }
    }

    // Trace write operations
    fn trace_write(location: &str, data: u8) {
        trace!("Writing to {}: 0x{:02x}", location, data)
    }
    fn trace_addr_write(location: &str, addr: u16, data: u8) {
        trace!("Writing to {}: 0x{:04x}, 0x{:02x}", location, addr, data)
    }

    // Write a value to the location
    fn write(&mut self, location: Location, data: u8) {
        match location {
            Location::Nowhere(addr) => error!("Attempted to write to nowhere in CPU: 0x{:04x}, 0x{:02x}.", addr, data),

            Location::Apu(addr) => {
                unsafe { *self.apu.get_unchecked_mut(addr as usize % APU_CAPACITY) = data }
                Self::trace_addr_write("APU", addr, data)
            }

            Location::CpuRam(addr) => {
                Self::trace_addr_write("CPU RAM", addr, data);
                self.cpu.write_ram(addr, data)
            }

            Location::PpuCtrl => {
                self.ppu.write_control(data);
                Self::trace_write("PPUCTRL", data);
            }

            Location::PpuMask => {
                self.ppu.write_mask(data);
                Self::trace_write("PPUCTRL", data);
            }

            Location::PpuStatus => {
                error!("Attempted to write to read only register PPUSTATUS: 0x{:02x}.", data)
            }

            Location::OamAddr => {
                self.ppu.write_oam_addr(data);
                Self::trace_write("OAMADDR", data);
            }

            Location::OamData => {
                let addr = self.ppu.oam_addr as u16;
                self.ppu.write_oam_data(data);
                Self::trace_addr_write("OAMDATA", addr, data);
            }

            Location::PpuAddr => {
                self.ppu.write_addr(data);
                Self::trace_write("PPUADDR", data);
            }

            Location::PpuScroll => {
                self.ppu.write_scroll(data);
                Self::trace_write("PPUSCROLL", data);
            }

            Location::PpuData => {
                self.ppu.write_data(data);
                Self::trace_write("PPUDATA", data);
            }

            Location::OamDma => {
                self.ppu.write_oam_dma(data);
                Self::trace_write("OAMDMA", data);
            }

            Location::PrgRam(addr) => {
                Self::trace_addr_write("PRG RAM", addr, data);
                self.cartridge.write_prg_ram(addr, data)
            }

            Location::PrgRom(addr) | Location::ChrRom(addr) => {
                error!("Attempted to write to read only memory in cartridge. {:04x?}, {:#02x}", location, data)
            }
        }
    }

    // Read an address on the CPU
    pub fn read_cpu(&mut self, addr: u16) -> u8 {
        let location = self.cartridge.cpu_read_location(addr);
        let data = self.read(location);

        data
    }

    // Read the address as a zero terminated string. Used mostly for testing.
    pub fn read_cpu_string(&mut self, addr: u16) -> String {
        let mut res = String::new();
        let mut addr = addr;
        loop {
            let ch = self.read_cpu(addr);
            if ch == 0 { break; }
            res.push(ch as char);
            addr += 1;
        }
        res
    }

    // Write into an address on the CPU
    pub fn write_cpu(&mut self, addr: u16, data: u8) {
        let location = self.cartridge.cpu_write_location(addr);
        self.write(location, data)
    }
}

impl fmt::Debug for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "{:?}\n", self.cpu)?;
        writeln!(formatter, "{:?}\n", self.ppu)?;
        writeln!(formatter, "{:?}\n", self.cartridge)?;
        write!(formatter, "APU | {:?}", (&self.apu[..]).hex_dump())
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
