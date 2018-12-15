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
    pub base_nametable_addr: u16,
    pub ram_increment: u16,
    pub sprite_pattern_table: u16,
    pub background_pattern_table: u16,
    pub sprite_size: u8,
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
    pub oam_addr: u8,
    pub oam_source: u16,

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
        let mut res = Self {
            latch: 0,

            // PPUCTRL
            base_nametable_addr: 0,
            ram_increment: 0,
            sprite_pattern_table: 0,
            background_pattern_table: 0,
            sprite_size: 0,
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
            oam_addr: 0,
            oam_source: 0,

            v: 0,
            w: false,

            ram: [0; RAM_CAPACITY],
            oam: [0; OAM_CAPACITY],
            ram_buffer: 0,
        };

        // Set defaults
        res.write_control(0);
        res.write_mask(0);
        res.write_scroll(0);

        res
    }

    // Peek PPUDATA
    pub fn peek_data(&self) -> u8 {
         unsafe { *self.ram.get_unchecked(self.get_addr()) }
    }

    // Read PPUDATA
    pub fn read_data(&mut self) -> u8 {
        let data = self.peek_data();

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

    // Peek PPUSTATUS
    pub fn peek_status(&self) -> u8 {
        // PPU status has some unused bits, so fill in from the latch.
        bits::copy(self.status, self.latch, 0b0001_1111)
    }

    // Read PPUSTATUS
    pub fn read_status(&mut self) -> u8 {
        // Reading the status resets the write toggle.
        self.w = false;

        // PPU status has some unused bits, so fill in from the latch.
        self.latch = self.peek_status();
        trace!("Reading from PPUSTATUS: 0x{:04x}", self.latch);

        // Vertical blank is cleared after reading status
        self.vblank_clear();

        self.latch
    }

    // Peek OAMDATA
    pub fn peek_oam_data(&self) -> u8 {
        unsafe { *self.oam.get_unchecked(self.oam_addr as usize) }
    }

    // Read OAMDATA
    pub fn read_oam_data(&mut self) -> u8 {
        self.latch = self.peek_oam_data();
        self.oam_addr = self.oam_addr.wrapping_add(1);
        self.latch
    }

    // Common routine for write operations
    pub fn write(&mut self, data: u8) { self.latch = data }

    // Write PPUCTRL
    pub fn write_control(&mut self, data: u8) {
        warn!("Writing into PPUCTRL is not completely implemented: 0x{:08b}", data);

        self.write(data);

        self.base_nametable_addr = match data & 0b0000_0011 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            x => unimplemented!(),
        };

        self.ram_increment = if bits::is_set(data, 2) { 32 } else { 1 };
        self.sprite_pattern_table = if bits::is_set(data, 3) { 0x1000 } else { 0x0000 };
        self.background_pattern_table = if bits::is_set(data, 4) { 0x1000 } else { 0x0000 };
        self.sprite_size = if bits::is_set(data, 5) { 16 } else { 8 };
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
        self.oam_addr = data;
    }

    // Write OAMDATA
    pub fn write_oam_data(&mut self, data: u8) {
        self.write(data);

        unsafe { *self.oam.get_unchecked_mut(self.oam_addr as usize) = data; }
        self.oam_addr = self.oam_addr.wrapping_add(1);
    }

    // Write OAMDMA
    pub fn write_oam_dma(&mut self, data: u8) {
        self.write(data);

        self.oam_transfer = true;
        self.oam_source = u16::from(data) << 8;
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
        writeln!(formatter, "PPU | {:?}\n", (&self.ram[..]).hex_dump())?;
        write!(formatter, "OAM | {:?}", (&self.oam[..]).hex_dump())
    }
}

impl Default for PpuData {
    fn default() -> Self { Self::new() }
}
