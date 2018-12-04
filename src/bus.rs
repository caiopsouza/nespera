use std::fmt;

use pretty_hex::*;

use crate::utils::bits;

// PPU registers
pub const PPU_CTRL: u16 = 0x2000;
pub const PPU_MASK: u16 = 0x2001;
pub const PPU_STATUS: u16 = 0x2002;
pub const OAM_ADDR: u16 = 0x2003;
pub const OAM_DATA: u16 = 0x2004;
pub const PPU_SCROLL: u16 = 0x2005;
pub const PPU_ADDR: u16 = 0x2006;
pub const PPU_DATA: u16 = 0x2007;
pub const OAM_DMA: u16 = 0x4014;

// Memory capacity
const RAM_CAPACITY: usize = 0x0800;
const ROM_CAPACITY: usize = 0x4000;
const APU_CAPACITY: usize = 0x0018;

// PPU capacity. Ends with the palette.
const PALETTE_SIZE: usize = 0x0020;
const PALETTE_START_POS: usize = 0x3f00;
const PPU_CAPACITY: usize = 0x4000;
const PPU_RAM_SIZE: u16 = 0x4000;

// Cpu data
#[derive(Clone)]
pub struct CpuData {
    pub ram: [u8; RAM_CAPACITY],
    pub rom: [u8; ROM_CAPACITY],
}

// Information about the PPU registers decoded from writing to them
#[derive(Clone)]
pub struct PpuData {
    // PPUCTRL
    pub ram_increment: u16,
    pub generate_nmi_at_vblank: bool,

    // PPUMASK
    pub greyscale: bool,
    pub show_background_in_lef: bool,
    pub show_sprites_in_leftmost: bool,
    pub show_background: bool,
    pub show_sprites: bool,
    pub emphasize_red: bool,
    pub emphasize_green: bool,
    pub emphasize_blue: bool,

    // PPUSTATUS
    pub status: u8,

    // PPUADDR
    pub addr: u8,

    // PPUDATA
    pub data: u8,

    // OAMADDR
    pub oam_transfer: bool,
    pub oam_dest: u8,
    pub oam_source: u8,

    // Internal PPU bus. Any read or write to its registers should fill it.
    pub latch: u8,

    // Internal registers
    // 15 bits. Current VRAM address.
    pub v: u16,

    // 1 bit. Write toggle.
    pub w: bool,

    // RAM
    pub ram: [u8; PPU_CAPACITY],
    pub ram_buffer: u8,
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
        if self.w {
            self.v = bits::set_low(self.v, addr);
        } else {
            self.v = bits::set_high(self.v, addr);
        }

        self.w = !self.w;

        // 15 bits only
        self.v &= 0b0111_1111_1111_1111;
    }

    // Check if an address refers to the palette region of memory
    fn is_palette(&self) -> bool { self.v as usize >= PALETTE_START_POS }

    // Increment the RAM address as specified by PPUCTRL
    fn inc_ram_addr(&mut self) {
        self.v = (self.v + self.ram_increment) % PPU_RAM_SIZE
    }

    // Get the RAM address as an index to the internal array
    fn get_addr(&self) -> usize {
        let mut addr = self.v as usize;

        // Palette is mirrored
        if self.is_palette() {
            addr = ((addr - PALETTE_START_POS) % PALETTE_SIZE) + PALETTE_START_POS
        }

        addr
    }

    // Read from memory as pointed by the internal address
    pub fn read_data(&mut self) -> u8 {
        let data = unsafe { *self.ram.get_unchecked(self.get_addr()) };

        // Palette data is read immediately
        let res = if self.is_palette() {
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
        println!("{:04x} {:02x}", addr, data);

        self.inc_ram_addr()
    }

    // Vertical blank flags
    pub fn vblank_set(&mut self) { self.status = bits::set(self.status, 7) }
    pub fn vblank_clear(&mut self) { self.status = bits::clear(self.status, 7) }

    // Sprite 0 hit flags
    pub fn sprite0_hit_set(&mut self) { self.status = bits::set(self.status, 6) }
    pub fn sprite0_hit_clear(&mut self) { self.status = bits::clear(self.status, 6) }

    // Sprite overflow flags
    pub fn overflow_set(&mut self) { self.status = bits::set(self.status, 5) }
    pub fn overflow_clear(&mut self) { self.status = bits::clear(self.status, 5) }

    // Reset OAMADDR
    pub fn oam_addr_reset(&mut self) { self.oam_dest = 0 }
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
    pub ppu: PpuData,
    pub apu: [u8; APU_CAPACITY],
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

                // PPUCTRL
                ram_increment: 1,
                generate_nmi_at_vblank: false,

                // PPUMASK
                greyscale: false,
                show_background_in_lef: false,
                show_sprites_in_leftmost: false,
                show_background: false,
                show_sprites: false,
                emphasize_red: false,
                emphasize_green: false,
                emphasize_blue: false,

                status: 0,
                addr: 0,
                data: 0,

                oam_transfer: false,
                oam_dest: 0,
                oam_source: 0,

                v: 0,
                w: false,

                ram: [0; PPU_CAPACITY],
                ram_buffer: 0,
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

    // Vblank has started
    pub fn start_vblank(&mut self) {
        self.ppu.vblank_set();
        if self.ppu.generate_nmi_at_vblank { self.nmi = true }
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
                    let ppu = &mut self.ppu;

                    // Reading from the PPU will fill the latch.
                    self.ppu.latch = match addr {
                        PPU_DATA => ppu.read_data(),

                        PPU_STATUS => {
                            // Reading the status resets the write toggle.
                            ppu.w = false;

                            // PPU status has some unused bits, so fill in from the latch.
                            let res = bits::copy(ppu.status, ppu.latch, 0b0001_1111);

                            // Vertical blank is cleared after reading
                            ppu.vblank_clear();

                            res
                        }

                        _ => {
                            println!("Warning: attempt to read ppu area not mapped: {:04X}.", addr);
                            self.ppu.latch
                        }
                    };

                    self.ppu.latch
                }

                // Areas not mapped.
                _ => {
                    unimplemented!("Warning: attempt to read bus area not mapped {:04X}.", addr);
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
                    let ppu = &mut self.ppu;

                    // Every write passes through the latch
                    ppu.latch = data;

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
                        PPU_CTRL => {
                            ppu.ram_increment = if bits::is_set(data, 2) { 32 } else { 1 };
                            ppu.generate_nmi_at_vblank = bits::is_set(data, 7);
                        }

                        PPU_MASK => {
                            ppu.greyscale = bits::is_set(data, 0);
                            ppu.show_background_in_lef = bits::is_set(data, 1);
                            ppu.show_sprites_in_leftmost = bits::is_set(data, 2);
                            ppu.show_background = bits::is_set(data, 3);
                            ppu.show_sprites = bits::is_set(data, 4);
                            ppu.emphasize_red = bits::is_set(data, 5);
                            ppu.emphasize_green = bits::is_set(data, 6);
                            ppu.emphasize_blue = bits::is_set(data, 7);
                        }

                        PPU_SCROLL => {}

                        PPU_ADDR => ppu.write_addr(data),

                        PPU_DATA => ppu.write_data(data),

                        OAM_ADDR => ppu.oam_dest = data,

                        _ => { panic!("Warning: attempt to write to ppu area not mapped: {:04X}.", addr); }
                    }
                }

                OAM_DMA => {
                    self.ppu.oam_transfer = true;
                    self.ppu.oam_source = data;
                }

                // Areas not mapped.
                _ => { panic!("Warning: attempt to write to bus area not mapped."); }
            }
        }
    }
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
