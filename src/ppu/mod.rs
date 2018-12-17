use std::cell::RefCell;
use std::intrinsics::assume;
use std::rc::Rc;
use std::time::Instant;

use crate::bus::Bus;
use crate::bus::ppu_data::SpriteSize;
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
        if cfg!(debug_assertions) {
            *self.screen.get(Self::screen_index(x, y)).unwrap()
        } else {
            *self.screen.get_unchecked(Self::screen_index(x, y))
        }
    }

    unsafe fn put_dot(&mut self, x: usize, y: usize, dot: u8) {
        if cfg!(debug_assertions) {
            *self.screen.get_mut(Self::screen_index(x, y)).unwrap() = dot
        } else {
            *self.screen.get_unchecked_mut(Self::screen_index(x, y)) = dot
        }
    }

    // Render all sprites from OAM.
    fn render_sprites(&mut self) {
        const CHUNKS_SIZE: usize = 4;

        let bus = self.bus.borrow();

        let sprite_table = bus.ppu.sprite_pattern_table;
        let sprite_size = bus.ppu.sprite_size;

        for sprite in bus.ppu.oam_chunks(CHUNKS_SIZE) {
            let y_sprite;
            let tile;
            let attr;
            let x_sprite;

            unsafe {
                assume(sprite.len() == CHUNKS_SIZE);

                y_sprite = sprite[0];
                tile = u16::from(sprite[1]);
                attr = sprite[2];
                x_sprite = sprite[3];
            }

            // Not visible
            if (0xef_u8..=0xff_u8).contains(&y_sprite) { continue; }

            // Pattern.
            // TODO: Don't assume the sprite is 8 pixels high.
            let addr = match sprite_size {
                SpriteSize::S8 => sprite_table + 0x10 * tile,
                SpriteSize::S16 => { 0x1000 * (tile & 1) + (0x10 * (tile >> 1)) }
            };

            // Palette
            let palette = bits::mask(attr, 0b_0011) as usize;

            // Flip the sprite
            let flip_x = bits::is_set(attr, 6);
            let flip_y = bits::is_set(attr, 7);

            for y in 0..8 {
                let y = if flip_y { 7 - y } else { y };

                for x in 0..8 {
                    let low = bus.cartridge.read_chr_rom(addr + y);
                    let high = bus.cartridge.read_chr_rom(addr + y + 8);
                    let pixel = bits::interlace(low, high, x);

                    // Transparent
                    if pixel == 0 { continue; }

                    let pixel = 0x3f10 + 0x04 * palette + pixel as usize;
                    let pixel = unsafe { bus.ppu.peek_ram(pixel) };

                    let x = if flip_x { 7 - x } else { x };

                    let x = (x_sprite).wrapping_add(x) as usize;
                    let y = (y_sprite).wrapping_add(y as u8) as usize;

                    // Out of bound
                    if x > SCREEN_WIDTH || y > SCREEN_HEIGHT { continue; }
                    unsafe { *self.screen.get_unchecked_mut(Self::screen_index(x, y)) = pixel }
                }
            }
        }
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
        let palette = Self::index(attribute_table, tile / 4, row / 4, 8);
        let palette = unsafe { bus.ppu.peek_ram(palette) };

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

                // Render all sprites.
                // TODO: Make this per pixel as the background.
                self.render_sprites();

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
