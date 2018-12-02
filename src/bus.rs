use std::fmt;

use pretty_hex::*;

use crate::utils::bits;

// PPU registers
const PPU_CTRL: u16 = 0x2000;
const PPU_MASK: u16 = 0x2001;
const PPU_STATUS: u16 = 0x2002;
// const OAM_ADDR: u16 = 0x2003;
const OAM_DATA: u16 = 0x2004;
const PPU_SCROLL: u16 = 0x2005;
const PPU_ADDR: u16 = 0x2006;
const PPU_DATA: u16 = 0x2007;
const OAM_DMA: u16 = 0x4014;

// Memory capacity
const RAM_CAPACITY: usize = 0x0800;
const ROM_CAPACITY: usize = 0x4000;
const APU_CAPACITY: usize = 0x0018;

// PPU capacity. Ends with the pallet
const PALLET_SIZE: usize = 0x0020;
const PALLET_START_POS: usize = 0x3f00;
const PPU_CAPACITY: usize = 0x4000;
const PPU_RAM_SIZE: u16 = 0x4000;

// Cpu data
#[derive(Clone)]
pub struct CpuData {
    pub ram: [u8; RAM_CAPACITY],
    pub rom: [u8; ROM_CAPACITY],
}

// Configuration flags
#[derive(Debug, Clone, PartialEq)]
enum AddrPos { High, Low }

// Information about the PPU registers decoded from writing to them
#[derive(Clone)]
pub struct PpuData {
    // Internal PPU bus. Any read or write to its registers should fill it.
    latch: u8,

    // PPUCTRL
    control: u8,

    // PPUMASK
    mask: u8,

    // PPUSTATUS
    status: u8,

    // PPUADDR
    addr: u8,

    // PPUDATA
    data: u8,

    // RAM
    ram: [u8; PPU_CAPACITY],
    ram_buffer: u8,
    ram_increment: u16,
    ram_addr_pos: AddrPos,
    ram_addr: u16,
}

impl PpuData {
    // Decode an address to its canonical value
    pub fn addr_decode(addr: u16) -> u16 {
        const PPU_REGS_CAPACITY: u16 = 0x08;
        const PPU_REGS_ADDR_START: u16 = 0x2000;
        (addr - PPU_REGS_ADDR_START) % PPU_REGS_CAPACITY + PPU_REGS_ADDR_START
    }

    // Write to the address register
    pub fn write_addr(&mut self, addr: u8) {
        if self.ram_addr_pos == AddrPos::Low {
            self.ram_addr = bits::set_low(self.ram_addr, addr);
            self.ram_addr_pos = AddrPos::High;
        } else {
            self.ram_addr = bits::set_high(self.ram_addr, addr);
            self.ram_addr_pos = AddrPos::Low;
        }

        self.ram_addr %= PPU_RAM_SIZE as u16;
    }

    // Check if an addres refers to the pallet region of memory
    fn is_pallet(&self) -> bool { self.ram_addr as usize >= PALLET_START_POS }

    // Increment the RAM address as specified by PPUCTRL
    fn inc_ram_addr(&mut self) {
        self.ram_addr = (self.ram_addr + self.ram_increment) % PPU_RAM_SIZE
    }

    // Get the RAM address as an index to the internal array
    fn get_addr(&self) -> usize {
        let mut addr = self.ram_addr as usize;

        // Pallet is mirrored
        if self.is_pallet() {
            addr = ((addr - PALLET_START_POS) % PALLET_SIZE) + PALLET_START_POS
        }

        addr
    }

    // Read from memory as pointed by the internal address
    pub fn read_data(&mut self) -> u8 {
        let data = unsafe { *self.ram.get_unchecked(self.get_addr()) };

        // Pallet data is read immediately
        let res = if self.is_pallet() {
            self.ram_buffer = data;
            data
        } else {
            let res = self.ram_buffer;
            self.ram_buffer = data;
            res
        };

        self.inc_ram_addr();
        res
    }

    // Write to memory as pointed by the internal address
    pub fn write_data(&mut self, data: u8) {
        let addr = self.get_addr();
        unsafe { *self.ram.get_unchecked_mut(addr) = data; }

        self.inc_ram_addr()
    }
}

// General communication between all parts of the NES
#[derive(Clone)]
pub struct Bus {
    // Interrupts
    pub reset: bool,
    pub nmi: bool,
    pub irq: bool,

    // Data
    pub cpu: CpuData,
    ppu: PpuData,
    apu: [u8; APU_CAPACITY],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            reset: true,
            nmi: false,
            irq: false,

            cpu: CpuData {
                ram: [0; RAM_CAPACITY],
                rom: [0; ROM_CAPACITY],
            },

            ppu: PpuData {
                latch: 0,

                control: 0,
                mask: 0,
                status: 0b10000000, // V blank
                addr: 0,
                data: 0,

                ram: [0; PPU_CAPACITY],
                ram_buffer: 0,
                ram_increment: 1,
                ram_addr_pos: AddrPos::High,
                ram_addr: 0,
            },

            apu: [0; APU_CAPACITY],
        }
    }

    pub fn with_mem(mem: Vec<u8>) -> Self {
        let mut res = Self::new();

        let ram_len = RAM_CAPACITY.min(mem.len());
        res.cpu.ram[..ram_len].copy_from_slice(&mem[..ram_len]);

        let rom_len = ROM_CAPACITY.min(mem.len());
        res.cpu.rom[..rom_len].copy_from_slice(&mem[..rom_len]);

        res
    }

    // Read an address
    pub fn read(&mut self, addr: u16) -> u8 {
        unsafe {
            match addr {
                // Ram
                0x0000...0x1fff =>
                    *self.cpu.ram.get_unchecked((addr - 0x0000) as usize % RAM_CAPACITY),

                // Rom
                0x8000...0xffff =>
                    *self.cpu.rom.get_unchecked((addr - 0x8000) as usize % ROM_CAPACITY),

                // Apu
                0x4000...0x4017 if addr != OAM_DMA =>
                    *self.apu.get_unchecked((addr - 0x4000) as usize % ROM_CAPACITY),

                // PPU
                0x2000...0x3FFF => {
                    let addr = PpuData::addr_decode(addr);

                    // Reading from the PPU will fill the latch.
                    self.ppu.latch = match addr {
                        OAM_DATA => { unimplemented!("Unable to read OAMDATA") }

                        PPU_DATA => self.ppu.read_data(),

                        // PPU status has some unused bits, so fill in from the latch.
                        PPU_STATUS => { bits::copy(self.ppu.status, self.ppu.latch, 0b0001_1111) }

                        // The rest are write only, so the result is whatever is on the latch.
                        _ => self.ppu.latch,
                    };

                    self.ppu.latch
                }

                // Areas not mapped.
                _ => {
                    panic!("Warning: attempt to read bus area not mapped.");
                    // 0
                }
            }
        }
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        unsafe {
            match addr {
                // Ram
                0x0000...0x1fff =>
                    *self.cpu.ram.get_unchecked_mut((addr - 0x0000) as usize % RAM_CAPACITY) = data,

                // Rom
                0x8000...0xffff =>
                    *self.cpu.rom.get_unchecked_mut((addr - 0x8000) as usize % ROM_CAPACITY) = data,

                // Apu
                0x4000...0x4017 if addr != OAM_DMA =>
                    *self.apu.get_unchecked_mut((addr - 0x4000) as usize % ROM_CAPACITY) = data,

                // PPU
                0x2000...0x3FFF => {
                    let addr = PpuData::addr_decode(addr);

                    // Every write passes through the latch
                    self.ppu.latch = data;

                    match addr {
                        //match addr {
                        // VPHB SINN
                        // |||| ||||
                        // |||| ||++- Base nametable address (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
                        // |||| |+--- VRAM address increment per CPU read/write of PPU_DATA (0: add 1, going across; 1: add 32, going down)
                        // |||| +---- Sprite pattern table address for 8x8 sprites (0: $0000; 1: $1000; ignored in 8x16 mode)
                        // |||+------ Background pattern table address (0: $0000; 1: $1000)
                        // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
                        // |+-------- PPU master/slave select (0: read backdrop from EXT pins; 1: output color on EXT pins)
                        // +--------- Generate an NMI at the start of the vertical blanking interval (0: off; 1: on)
                        //}
                        PPU_CTRL => { self.ppu.ram_increment = if bits::is_set(data, 2) { 32 } else { 1 }; }

                        PPU_MASK | PPU_SCROLL => {}

                        PPU_ADDR => self.ppu.write_addr(data),

                        PPU_DATA => self.ppu.write_data(data),

                        _ => panic!("write ppu | addr: {:04x}, data: {:02x}", addr, data),
                    }
                }

                // Areas not mapped.
                _ => {
                    panic!("Warning: attempt to write to bus area not mapped.");
                }
            }
        }
    }

    // Peek and poke should not have side effects besides changing the data on poke
}

impl fmt::Debug for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "RAM | {:?}", (&self.cpu.ram[..]).hex_dump())?;
        writeln!(formatter, "ROM | {:?}", (&self.cpu.rom[..]).hex_dump())?;
        writeln!(formatter, "PPU | {:?}", (&self.ppu.ram[..]).hex_dump())?;
        write!(formatter, "APU | {:?}", (&self.apu[..]).hex_dump())
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

impl PartialEq for Bus {
    fn eq(&self, other: &Bus) -> bool {
        self.cpu.ram.eq(&other.cpu.ram[..])
            && self.cpu.rom.eq(&other.cpu.rom[..])
            && self.apu.eq(&other.apu[..])
    }
}
