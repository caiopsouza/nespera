use std::fmt;
use std::slice::Chunks;

use pretty_hex::PrettyHex;

use crate::utils::bits;
use crate::cartridge::Mirror;

const NAMETABLE_BASE: usize = 0x2000;

// PPU capacity. Ends with the palette.
pub const PALETTE_CAPACITY: usize = 0x0020;
pub const PALETTE_START_POS: usize = 0x3f00;
const PPU_CAPACITY: usize = 0x4000;
const RAM_CAPACITY: usize = 0x4000;
const OAM_CAPACITY: usize = 0x0100;

#[derive(Debug, Copy, Clone)]
pub enum SpriteSize { S8, S16 }

// T and V are composed this way during rendering:
// yyy NN YYYYY XXXXX
// ||| || ||||| +++++-- coarse X scroll
// ||| || +++++-------- coarse Y scroll
// ||| ++-------------- nametable select
// +++----------------- fine Y scroll
#[derive(Debug)]
pub struct VRamAddr {
    pub coarse_x: u16,
    pub coarse_y: u16,
    pub horizontal_nametable: bool,
    pub vertical_nametable: bool,
    pub fine_y: u16,
}

impl VRamAddr {
    pub fn new(value: u16) -> Self {
        Self {
            coarse_x: (value & 0b000_00_00000_11111),
            coarse_y: (value & 0b000_00_11111_00000) >> 5,
            horizontal_nametable: (value & 0b000_01_00000_00000) != 0,
            vertical_nametable: (value & 0b000_10_00000_00000) != 0,
            fine_y: (value & 0b111_00_00000_00000) >> 12,
        }
    }

    pub fn as_u16(&self) -> u16 {
        debug_assert!(self.coarse_x <= 0b11111);
        debug_assert!(self.coarse_y <= 0b11111);
        debug_assert!(self.fine_y <= 0b111);

        self.coarse_x
            | self.coarse_y << 5
            | (u16::from(self.horizontal_nametable)) << 10
            | (u16::from(self.vertical_nametable)) << 11
            | self.fine_y << 12
    }

    pub fn inc_coarse_x(&mut self) {
        if self.coarse_x < 31 {
            self.coarse_x += 1;
            return;
        }

        self.coarse_x = 0;
        self.horizontal_nametable = !self.horizontal_nametable;
    }

    pub fn inc_fine_y(&mut self) {
        if self.fine_y < 7 {
            self.fine_y += 1;
            return;
        }

        self.fine_y = 0;

        // Increment coarse Y
        if self.coarse_y == 29 {
            self.coarse_y = 0;
            self.vertical_nametable = !self.vertical_nametable;
        } else if self.coarse_y == 31 {
            self.coarse_y = 0;
        } else {
            self.coarse_y += 1;
        }
    }
}

// Information about the PPU registers decoded from writing to them
pub struct PpuData {
    // PPUCTRL
    pub base_nametable_addr: usize,
    pub ram_increment: u16,
    pub sprite_pattern_table: u16,
    pub background_pattern_table: u16,
    pub sprite_size: SpriteSize,
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
    // Current VRAM address. 15 bits.
    pub v: u16,

    // Temporary VRAM address. 15 bits.
    // Can also be thought of as the address of the top left onscreen tile.
    pub t: u16,

    // Fine X scroll. 3 bits.
    pub x: u8,

    // Write toggle. 1 bit.
    pub w: bool,

    // RAM
    ram: [u8; RAM_CAPACITY],
    oam: [u8; OAM_CAPACITY],
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
            sprite_size: SpriteSize::S8,
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
            t: 0,
            x: 0,
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

    // Direct RAM and OAM access.
    pub unsafe fn peek_ram(&self, addr: usize) -> u8 {
        debug_assert!(addr < self.ram.len(), "PPU RAM out of bounds: {}", addr);
        *self.ram.get_unchecked(addr)
    }

    pub unsafe fn peek_oam(&self, addr: usize) -> u8 {
        debug_assert!(addr < self.oam.len(), "PPU OAM out of bounds: {}", addr);
        *self.oam.get_unchecked(addr)
    }

    pub fn oam_chunks(&self, size: usize) -> Chunks<u8> { self.oam.chunks(size) }

    unsafe fn poke_ram(&mut self, addr: usize, data: u8) {
        debug_assert!(addr < self.ram.len(), "PPU RAM out of bounds: {}", addr);
        *self.ram.get_unchecked_mut(addr) = data
    }

    unsafe fn poke_oam(&mut self, addr: usize, data: u8) {
        debug_assert!(addr < self.oam.len(), "PPU OAM out of bounds: {}", addr);
        *self.oam.get_unchecked_mut(addr) = data
    }

    // Vram attributes
    pub fn get_fine_y(&self) -> u16 { VRamAddr::new(self.v).fine_y }

    pub fn inc_coarse_x(&mut self) {
        let mut v = VRamAddr::new(self.v);
        v.inc_coarse_x();
        self.v = v.as_u16();
    }

    pub fn inc_fine_y(&mut self) {
        let mut v = VRamAddr::new(self.v);
        v.inc_fine_y();
        self.v = v.as_u16();
    }

    pub fn copy_horizontal_v(&mut self) {
        let t = VRamAddr::new(self.t);
        let mut v = VRamAddr::new(self.v);
        v.coarse_x = t.coarse_x;
        v.horizontal_nametable = t.horizontal_nametable;
        self.v = v.as_u16();
    }

    pub fn copy_vertical_v(&mut self) {
        let t = VRamAddr::new(self.t);
        let mut v = VRamAddr::new(self.v);
        v.coarse_y = t.coarse_y;
        v.vertical_nametable = t.vertical_nametable;
        v.fine_y = t.fine_y;
        self.v = v.as_u16();
    }

    pub fn fetch_nametable(&self, mirror: Mirror) -> u8 {
        let mut v = VRamAddr::new(self.v);
        v.fine_y = 0;

        if mirror == Mirror::Vertical {
            v.vertical_nametable = false
        } else {
            v.horizontal_nametable = false
        }

        let addr = NAMETABLE_BASE | (v.as_u16() as usize);
        unsafe { self.peek_ram(addr) }
    }

    pub fn fetch_attribute(&self) -> u16 {
        // Address of attribute is composed like so:
        // NN 1111 YYY XXX
        // || |||| ||| +++-- high 3 bits of coarse X (x/4)
        // || |||| +++------ high 3 bits of coarse Y (y/4)
        // || ++++---------- attribute offset (960 bytes)
        // ++--------------- nametable select
        let v = VRamAddr::new(self.v);
        let addr = NAMETABLE_BASE
            | ((v.vertical_nametable as usize) << 12)
            | ((v.horizontal_nametable as usize) << 11)
            | 0b00_1111_000_000
            | ((v.coarse_y as usize >> 2) << 3)
            | (v.coarse_x as usize >> 2)
            ;
        unsafe { self.peek_ram(addr) as u16 }
    }

    // Peek PPUDATA
    pub fn peek_data(&self) -> u8 { unsafe { self.peek_ram(self.get_addr()) } }

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

        self.latch = self.peek_status();

        // Vertical blank is cleared after reading status
        self.vblank_clear();

        self.latch
    }

    // Peek OAMDATA
    pub fn peek_oam_data(&self) -> u8 { unsafe { self.peek_oam(self.oam_addr as usize) } }

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
        self.write(data);

        self.base_nametable_addr = match data & 0b0000_0011 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2c00,
            x => unimplemented!(),
        };

        let mut t = VRamAddr::new(self.t);
        t.horizontal_nametable = bits::is_set(data, 0);
        t.vertical_nametable = bits::is_set(data, 1);
        self.t = t.as_u16();

        self.ram_increment = if bits::is_set(data, 2) { 32 } else { 1 };
        self.sprite_pattern_table = if bits::is_set(data, 3) { 0x1000 } else { 0x0000 };
        self.background_pattern_table = if bits::is_set(data, 4) { 0x1000 } else { 0x0000 };
        self.sprite_size = if bits::is_set(data, 5) { SpriteSize::S16 } else { SpriteSize::S8 };
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
        let data = u16::from(data);

        let fine = data & 0b000000_0111;
        let coarse = data & 0b1111_1000 >> 3;

        let mut t = VRamAddr::new(self.t);

        if self.w {
            t.coarse_y = coarse;
            t.fine_y = fine;
        } else {
            t.coarse_x = coarse;
            self.x = fine as u8; // Fine X is read from its own register.
        }

        self.t = t.as_u16();
        self.w = !self.w;
    }

    // Write PPUADDR
    pub fn write_addr(&mut self, data: u8) {
        self.write(data);

        if self.w {
            self.t = bits::set_low(self.t, data);
            self.v = self.t;
        } else {
            // Address has 14 bits at most.
            self.t = bits::set_high(self.t, data) & 0b0011_1111_1111_1111;
        }

        self.w = !self.w;
    }

    // Write PPUDATA
    pub fn write_data(&mut self, data: u8) {
        self.write(data);

        let addr = self.get_addr();
        unsafe { self.poke_ram(addr, data) }

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

        unsafe { self.poke_oam(self.oam_addr as usize, data) }
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
