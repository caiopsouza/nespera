use std::fmt;

use pretty_hex::PrettyHex;

use crate::utils::bits;

// PPU registers
const PPU_CTRL: u16 = 0x2000;
const PPU_MASK: u16 = 0x2001;
const PPU_STATUS: u16 = 0x2002;
const OAM_ADDR: u16 = 0x2003;
const OAM_DATA: u16 = 0x2004;
// const PPU_SCROLL: u16 = 0x2005;
const PPU_ADDR: u16 = 0x2006;
const PPU_DATA: u16 = 0x2007;
pub const OAM_DMA: u16 = 0x4014;

// PPU capacity. Ends with the palette.
const PALETTE_SIZE: usize = 0x0020;
const PALETTE_START_POS: usize = 0x3f00;
const RAM_CAPACITY: usize = 0x4000;
const OAM_CAPACITY: usize = 0x0100;

// Information about the PPU registers decoded from writing to them
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
    pub ram: [u8; RAM_CAPACITY],
    pub oam: [u8; OAM_CAPACITY],
    pub ram_buffer: u8,
}

impl PpuData {
    pub fn new() -> Self {
        Self {
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

            ram: [0; RAM_CAPACITY],
            oam: [0; OAM_CAPACITY],
            ram_buffer: 0,
        }
    }

    // Read value from the PPU internal memory
    pub fn read(&mut self, addr: u16) -> u8 {
        let addr = PpuData::addr_decode(addr);

        // Reading from the PPU will fill the latch.
        self.latch = match addr {
            PPU_DATA => {
                let data = unsafe { *self.ram.get_unchecked(self.get_addr()) };

                // Palette data is read immediately.
                // Everything else is read into a buffer and the previous contents of the buffer is returned.
                let res = if self.is_palette() {
                    self.ram_buffer = data;
                    data
                } else {
                    let res = self.ram_buffer;
                    self.ram_buffer = data;
                    res
                };

                self.inc_ram_addr();

                trace!("Reading from PPUDATA: 0x{:04x}", res);
                res
            }

            PPU_STATUS => {

                // Reading the status resets the write toggle.
                self.w = false;

                // PPU status has some unused bits, so fill in from the latch.
                let res = bits::copy(self.status, self.latch, 0b0001_1111);
                trace!("Reading from PPUSTATUS: 0x{:04x}", res);

                // Vertical blank is cleared after reading status
                self.vblank_clear();

                res
            }

            _ => {
                warn!("Reading from write only PPU area. Addr 0x{:04x}.", addr);
                self.latch
            }
        };

        self.latch
    }

    // Write value into the PPU internal memory
    pub fn write(&mut self, addr: u16, data: u8) {
        let addr = PpuData::addr_decode(addr);

        // Every write passes through the latch
        self.latch = data;

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
                warn!("Writing into PPUCTRL is not completely implemented: 0x{:02x}", data);

                self.ram_increment = if bits::is_set(data, 2) { 32 } else { 1 };
                self.generate_nmi_at_vblank = bits::is_set(data, 7);
            }

            PPU_MASK => {
                trace!("Writing into PPUMASK: 0x{:02x}", data);

                self.greyscale = bits::is_set(data, 0);
                self.show_background_in_lef = bits::is_set(data, 1);
                self.show_sprites_in_leftmost = bits::is_set(data, 2);
                self.show_background = bits::is_set(data, 3);
                self.show_sprites = bits::is_set(data, 4);
                self.emphasize_red = bits::is_set(data, 5);
                self.emphasize_green = bits::is_set(data, 6);
                self.emphasize_blue = bits::is_set(data, 7);
            }

            PPU_ADDR => {
                trace!("Writing into PPUADDR: 0x{:02x}", data);

                if self.w {
                    self.v = bits::set_low(self.v, data);
                } else {
                    self.v = bits::set_high(self.v, data);
                }

                self.w = !self.w;

                // 15 bits only
                self.v &= 0b0111_1111_1111_1111;
            }

            PPU_DATA => {
                trace!("Writing into PPUDATA: 0x{:02x}", data);

                let addr = self.get_addr();
                unsafe { *self.ram.get_unchecked_mut(addr) = data; }

                self.inc_ram_addr()
            }

            OAM_ADDR => {
                trace!("Writing into OAMADDR: 0x{:02x}", data);
                self.oam_dest = data;
            }

            OAM_DATA => {
                trace!("Writing into OAMDATA: 0x{:02x}", data);

                unsafe { *self.oam.get_unchecked_mut(self.oam_dest as usize) = data; }
                self.oam_dest = self.oam_dest.wrapping_add(1);
            }

            OAM_DMA => {
                trace!("Writing into OAMDMA: 0x{:02x}", data);

                self.oam_transfer = true;
                self.oam_source = data;
            }

            _ => {
                warn!("Writing to PPU area not mapped. Addr 0x{:04x}. Data 0x{:02x}.", addr, data);
            }
        }
    }

    // Decode an address to its canonical value
    fn addr_decode(addr: u16) -> u16 {
        const PPU_REGS_CAPACITY: u16 = 0x08;
        const PPU_REGS_ADDR_START: u16 = 0x2000;
        (addr - PPU_REGS_ADDR_START) % PPU_REGS_CAPACITY + PPU_REGS_ADDR_START
    }

    // Check if an address refers to the palette region of memory
    fn is_palette(&self) -> bool { self.v as usize >= PALETTE_START_POS }

    // Increment the RAM address as specified by PPUCTRL
    fn inc_ram_addr(&mut self) {
        self.v = (self.v + self.ram_increment) % (RAM_CAPACITY as u16)
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

    // Vertical blank flags
    pub fn vblank_set(&mut self) {
        trace!("vblank set");
        self.status = bits::set(self.status, 7)
    }

    pub fn vblank_clear(&mut self) {
        trace!("vblank clear");
        self.status = bits::clear(self.status, 7)
    }
}

impl fmt::Debug for PpuData {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "PPU | {:?}", (&self.ram[..]).hex_dump())
    }
}
