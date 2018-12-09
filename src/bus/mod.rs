#![allow(unused_variables)]

use std::fmt;

use pretty_hex::PrettyHex;

use crate::bus::cpu_data::CpuData;
use crate::bus::ppu_data::PpuData;
use crate::mapper::Cartridge;

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
        Self::create(CpuData::with_mem(mem), Cartridge::empty())
    }

    pub fn with_cartridge(cartridge: Cartridge) -> Self {
        Self::create(CpuData::new(), cartridge)
    }

    // Vblank has started
    pub fn start_vblank(&mut self) {
        self.ppu.vblank_set();
        if self.ppu.generate_nmi_at_vblank { self.nmi = true }
    }

    // Read an address
    pub fn read(&mut self, addr: u16) -> u8 {
        unsafe {
            match addr {
                0x0000...0x1fff => self.cpu.read_ram(addr),

                0x2000...0x3fff => self.ppu.read(addr),

                ppu_data::OAM_DMA => self.ppu.read(addr),

                0x4000...0x4017 => {
                    warn!("Reading from APU address 0x{:4x}.", addr);
                    *self.apu.get_unchecked((addr - 0x4000) as usize % APU_CAPACITY)
                }

                0x4020...0xffff => self.cartridge.read(addr),

                _ => {
                    error!("Reading from area not mapped. Addr 0x{:04x}.", addr);
                    0
                }
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000...0x1fff => self.cpu.write_ram(addr, data),

            ppu_data::OAM_DMA => self.ppu.write(addr, data),

            0x2000...0x3FFF => self.ppu.write(addr, data),

            0x4000...0x4017 => {
                warn!("Writing into APU address: {:#04x}, data: {:#02x}.", addr, data);
                unsafe { *self.apu.get_unchecked_mut((addr - 0x4000) as usize % APU_CAPACITY) = data; }
            }

            0x4020...0xffff => self.cartridge.write(addr, data),

            // Areas not mapped.
            _ => error!("Writing into area not mapped. Addr {:#04x}. Data: {:#02x}", addr, data),
        }
    }
}

impl fmt::Debug for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "{:?}", self.cpu)?;
        writeln!(formatter, "{:?}", self.ppu)?;
        write!(formatter, "APU | {:?}", (&self.apu[..]).hex_dump())
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
