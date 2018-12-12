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

    // Read an address on the CPU
    pub fn read_cpu(&mut self, addr: u16) -> u8 {
        unsafe {
            match addr {
                0x0000...0x1fff => {
                    let res = self.cpu.read_ram(addr);
                    trace!("Reading from CPU RAM: 0x{:04x}, 0x{:02x}", addr, res);
                    res
                }

                0x2000...0x3fff => self.ppu.read_register(addr),

                ppu_data::OAM_DMA => self.ppu.read_register(addr),

                0x4000...0x4017 => {
                    warn!("Reading from APU address 0x{:4x}.", addr);
                    *self.apu.get_unchecked((addr - 0x4000) as usize % APU_CAPACITY)
                }

                0x4020...0xffff => self.cartridge.read_cpu(addr),

                _ => {
                    error!("Reading from area not mapped in CPU. Addr 0x{:04x}.", addr);
                    0
                }
            }
        }
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
        match addr {
            0x0000...0x1fff => {
                trace!("Writing into CPU RAM: 0x{:04x}, 0x{:02x}", addr, data);
                self.cpu.write_ram(addr, data);
            }

            ppu_data::OAM_DMA => self.ppu.write_register(addr, data),

            0x2000...0x3FFF => self.ppu.write_register(addr, data),

            0x4000...0x4017 => {
                warn!("Writing into APU address: {:#04x}, data: {:#02x}.", addr, data);
                unsafe { *self.apu.get_unchecked_mut((addr - 0x4000) as usize % APU_CAPACITY) = data; }
            }

            0x4020...0xffff => self.cartridge.write_cpu(addr, data),

            // Areas not mapped.
            _ => error!("Writing into area not mapped in CPU. Addr {:#04x}. Data: {:#02x}", addr, data),
        }
    }
}

impl fmt::Debug for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "{:?}", self.cpu)?;
        writeln!(formatter, "{:?}", self.ppu)?;
        writeln!(formatter, "{:?}", self.cartridge)?;
        write!(formatter, "APU | {:?}", (&self.apu[..]).hex_dump())
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}
