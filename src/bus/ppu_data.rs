use std::fmt;

use pretty_hex::PrettyHex;

use crate::utils::bits;

// PPU capacity. Ends with the palette.
pub const PALETTE_CAPACITY: usize = 0x0020;
pub const PALETTE_START_POS: usize = 0x3f00;
const PPU_CAPACITY: usize = 0x4000;
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
    pub palette: [u8; PALETTE_CAPACITY],
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
            palette: [0; PALETTE_CAPACITY],
            oam: [0; OAM_CAPACITY],
            ram_buffer: 0,
        }
    }

    // Read PPUDATA
    pub fn read_data(&mut self) -> u8 {
        let data = unsafe { *self.ram.get_unchecked(self.get_addr()) };

        // Palette data is read immediately.
        // Everything else is read into a buffer and the previous contents of the buffer is returned.
        self.latch = if self.is_palette() {
            self.ram_buffer = data;
            data
        } else {
            let res = self.ram_buffer;
            self.ram_buffer = data;
            res
        };

        self.inc_ram_addr();

        trace!("Reading from PPUDATA: 0x{:04x}", self.latch);
        self.latch
    }

    // Read PPUSTATUS
    pub fn read_status(&mut self) -> u8 {
        // Reading the status resets the write toggle.
        self.w = false;

        // PPU status has some unused bits, so fill in from the latch.
        self.latch = bits::copy(self.status, self.latch, 0b0001_1111);
        trace!("Reading from PPUSTATUS: 0x{:04x}", self.latch);

        // Vertical blank is cleared after reading status
        self.vblank_clear();

        self.latch
    }

    // Common routine for write operations
    pub fn write(&mut self, data: u8) { self.latch = data }

    // Write PPUCONTROL
    pub fn write_control(&mut self, data: u8) {
        warn!("Writing into PPUCTRL is not completely implemented: 0x{:02x}", data);

        self.write(data);

        self.ram_increment = if bits::is_set(data, 2) { 32 } else { 1 };
        self.generate_nmi_at_vblank = bits::is_set(data, 7);
    }

    // Write PPUMASK
    pub fn write_mask(&mut self, data: u8) {
        self.write(data);

        self.greyscale = bits::is_set(data, 0);
        self.show_background_in_lef = bits::is_set(data, 1);
        self.show_sprites_in_leftmost = bits::is_set(data, 2);
        self.show_background = bits::is_set(data, 3);
        self.show_sprites = bits::is_set(data, 4);
        self.emphasize_red = bits::is_set(data, 5);
        self.emphasize_green = bits::is_set(data, 6);
        self.emphasize_blue = bits::is_set(data, 7);
    }

    // Write PPUSCROLL
    pub fn write_scroll(&mut self, data: u8) {
        self.write(data);
        error!("Writing to PPUSCROLL is not implemented.");
    }

    // Write PPUADDR
    pub fn write_addr(&mut self, data: u8) {
        self.write(data);

        if self.w {
            self.v = bits::set_low(self.v, data);
        } else {
            self.v = bits::set_high(self.v, data);
        }

        self.w = !self.w;

        // 15 bits only
        self.v &= 0b0111_1111_1111_1111;
    }

    // Write PPUDATA
    pub fn write_data(&mut self, data: u8) {
        self.write(data);

        let addr = self.get_addr();
        unsafe { *self.ram.get_unchecked_mut(addr) = data; }

        self.inc_ram_addr()
    }

    // Write OAMADDR
    pub fn write_oam_addr(&mut self, data: u8) {
        self.write(data);
        self.oam_dest = data;
    }

    // Write OAMDATA
    pub fn write_oam_data(&mut self, data: u8) {
        self.write(data);

        unsafe { *self.oam.get_unchecked_mut(self.oam_dest as usize) = data; }
        self.oam_dest = self.oam_dest.wrapping_add(1);
    }

    // Write OAMDMA
    pub fn write_oam_dma(&mut self, data: u8) {
        self.write(data);

        self.oam_transfer = true;
        self.oam_source = data;
    }

    // Check if an address refers to the palette region of memory
    fn is_palette(&self) -> bool { self.v as usize >= PALETTE_START_POS }

    // Increment the RAM address as specified by PPUCTRL
    fn inc_ram_addr(&mut self) {
        self.v = (self.v + self.ram_increment) % (PPU_CAPACITY as u16)
    }

    // Get the RAM address as an index to the internal array
    fn get_addr(&self) -> usize {
        let mut addr = self.v as usize;

        // Palette is mirrored
        if self.is_palette() {
            addr = ((addr - PALETTE_START_POS) % PALETTE_CAPACITY) + PALETTE_START_POS
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
