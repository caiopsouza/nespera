use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::bus::Bus;
use crate::utils::bits;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub struct Ppu {
    pub clock: u32,

    // Address Bus
    pub bus: Rc<RefCell<Bus>>,

    // Rendering information
    pub frame: u32,
    pub scanline: i32,
    pub dot: u32,

    // Fps calculation
    frame_start: Instant,
    pub fps: f64,

    // Screen result
    pub screen: [u8; SCREEN_SIZE],
}

impl Ppu {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        Self {
            clock: 0,
            bus,
            frame: 0,
            scanline: 0,
            dot: 0,
            frame_start: Instant::now(),
            fps: 0_f64,
            screen: [0; SCREEN_SIZE],
        }
    }

    // Matrix indexes
    fn index(base: usize, x: usize, y: usize, width: usize) -> usize { base + x + y * width }
    fn screen_index(x: usize, y: usize) -> usize { Self::index(0, x, y, SCREEN_WIDTH) }

    // Screen operations
    pub unsafe fn get_dot(&mut self, x: usize, y: usize) -> u8 {
        *self.screen.get_unchecked(Self::screen_index(x, y))
    }

    unsafe fn put_dot(&mut self, x: usize, y: usize, dot: u8) {
        *self.screen.get_unchecked_mut(Self::screen_index(x, y)) = dot
    }

    // Render the current dot
    pub fn render(&mut self) {
        let scanline = self.scanline as usize;
        let dot = self.dot as usize;

        // Not visible in these cases.
        if !(0..SCREEN_HEIGHT).contains(&scanline) { return; }
        if !(0..SCREEN_WIDTH).contains(&dot) { return; }

        let bus = self.bus.borrow_mut();

        // Tables
        let background_table = bus.ppu.base_nametable_addr as usize;
        let attribute_table = background_table + 0x03c0;
        let pattern_table = bus.ppu.background_pattern_table;

        // Position on the name table.
        let row = scanline / 8;
        let tile = dot / 8;

        // Background
        let background = Self::index(background_table, tile, row, 32);
        let pattern = pattern_table + u16::from(unsafe { bus.ppu.peek_ram(background) }) * 0x10;

        // Palette
        let palette_addr = Self::index(attribute_table, tile / 4, row / 4, 8);
        let palette = unsafe { bus.ppu.peek_ram(palette_addr) };

        // Position inside the pattern.
        let x = dot as u8 % 8;
        let y = scanline as u16 % 8;

        // Pixel
        let low = bus.cartridge.read_chr_rom(pattern + y);
        let high = bus.cartridge.read_chr_rom(pattern + y + 8);
        let pixel = bits::interlace(low, high, x);

        let pixel = if pixel == 0 {
            0_u8
        } else {
            let mask = 2 * (((row as u8 & 1) << 1) | (tile as u8 & 1));
            let color = (palette & (0b0000_0011 << mask)) >> mask;

            (color << 2) | pixel
        };

        let pixel = 0x3f00 + pixel as usize;
        let pixel = unsafe { bus.ppu.peek_ram(pixel) };

        drop(bus);
        unsafe { self.put_dot(dot, scanline, pixel) }
    }

    // Run until some condition is met
    pub fn step(&mut self) {
        // Render the dot
        let is_rendering = {
            let bus = self.bus.borrow();
            bus.ppu.show_background || bus.ppu.show_sprites
        };
        if is_rendering { self.render() }

        // Increment the clock, dot and scanline
        self.clock += 1;
        self.dot += 1;

        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline == 0 {
                // On odd frames this dot is skipped if rendering is enabled.
                if is_rendering && self.frame % 2 == 1 { self.dot += 1 }
            } else if self.scanline > 260 {
                trace!("Finished running frame {}.", self.frame);

                self.scanline = -1;
                self.frame += 1;

                // Update fps
                let now = Instant::now();
                self.fps = 1_f64 / (now - self.frame_start).as_float_secs();
                self.frame_start = now;
            }
        }

        // Vblank
        if self.dot == 0 {
            match self.scanline {
                -1 => self.bus.borrow_mut().ppu.vblank_clear(),
                241 => self.bus.borrow_mut().start_vblank(),
                _ => {}
            }
        }
    }
}
