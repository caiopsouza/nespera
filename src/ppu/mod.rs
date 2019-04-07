use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::bus::Bus;
use crate::utils::bits;
use crate::bus::ppu_data::SpriteSize;
use std::intrinsics::assume;
use crate::bus::ppu_data::VRamAddr;

pub const SCREEN_WIDTH: usize = 256;
pub const SCREEN_HEIGHT: usize = 240;
pub const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

#[derive(Copy, Clone)]
struct RenderInfo {
    background: [u8; 8],
    attribute: u16,
}

pub struct Ppu {
    pub clock: u32,

    // Address Bus
    pub bus: Rc<RefCell<Bus>>,

    // Rendering information
    pub frame: u32,
    pub scanline: i32,
    pub dot: u32,

    // Rendering data.
    render: [RenderInfo; 2],
    name_table: u16,
    attribute: u16,
    low_background: u8,
    high_background: u8,

    // Fps calculation
    frame_start: Instant,
    pub fps: f64,

    // Screen result
    pub screen: [u8; SCREEN_SIZE],
}

impl Ppu {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        let render = RenderInfo {
            background: [0; 8],
            attribute: 0,
        };

        Self {
            clock: 0,
            bus,

            frame: 1,
            scanline: 0,
            dot: 30,

            render: [render; 2],

            name_table: 0,
            attribute: 0,
            low_background: 0,
            high_background: 0,

            frame_start: Instant::now(),
            fps: 0_f64,
            screen: [0; SCREEN_SIZE],
        }
    }

    // Matrix indexes
    fn index(base: usize, x: usize, y: usize, width: usize) -> usize { base + x + y * width }
    fn screen_index(x: usize, y: usize) -> usize { Self::index(0, x, y, SCREEN_WIDTH) }

    unsafe fn put_dot_on_screen(screen: &mut [u8; SCREEN_SIZE], x: usize, y: usize, dot: u8) {
        debug_assert!(x < SCREEN_WIDTH && y < SCREEN_HEIGHT, "Screen point out of bounds. x: {}, y: {}", x, y);
        *screen.get_unchecked_mut(Self::screen_index(x, y)) = dot
    }

    unsafe fn put_dot(&mut self, x: usize, y: usize, dot: u8) {
        Self::put_dot_on_screen(&mut self.screen, x, y, dot)
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
                    let pixel = bits::interlace(low, high)[x as usize];

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
        let background_table = bus.ppu.base_nametable_addr;
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
        let x = dot % 8;
        let y = scanline as u16 % 8;

        // Pixel
        let low = bus.cartridge.read_chr_rom(pattern + y);
        let high = bus.cartridge.read_chr_rom(pattern + y + 8);
        let pixel = bits::interlace(low, high)[x];

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

    // Run one step on the PPU.
    pub fn step(&mut self) {
        let mut bus = self.bus.borrow_mut();
        let (data, cartridge) = bus.get_ppu_and_cartridge();
        let v = VRamAddr::new(data.v);

        // Various PPU states.
        let rendering_enabled = data.show_background || data.show_sprites;
        let show_background = data.show_background;

        let fetch_scanline = self.scanline < 240;
        let fetch_dot = (1..257).contains(&self.dot) || (321..341).contains(&self.dot);

        let visible_scanline = (0..240).contains(&self.scanline);
        let visible_dot = (0..256).contains(&self.dot);

        let copy_vertical_scanline = self.scanline == -1;
        let copy_vertical_dot = (280..305).contains(&self.dot);

        // Fetch data.
        if rendering_enabled && fetch_scanline && fetch_dot {
            // Each fetch takes two cycles starting at dot 1.
            match self.dot % 8 {
                1 => {
                    let name_table = u16::from(data.fetch_nametable(cartridge.ppu_mirror));
                    self.name_table = data.background_pattern_table + (name_table << 4);
                }
                3 => self.attribute = data.fetch_attribute(),
                5 => self.low_background = cartridge.read_chr_rom(self.name_table + v.fine_y),
                7 => self.high_background = cartridge.read_chr_rom(self.name_table + v.fine_y + 8),
                0 => {
                    data.inc_coarse_x();
                    self.render[0] = self.render[1];
                    self.render[1].background = bits::interlace(self.low_background, self.high_background);
                    self.render[1].attribute = self.attribute;
                }
                _ => {}
            }
        }

        // Render background
        if show_background && visible_scanline && visible_dot {
            let scanline = self.scanline as usize;
            let dot = self.dot as usize;
            let render = self.render[0];

            // Pixel
            let pixel = render.background[dot % 8 + data.x as usize];
            let pixel = if pixel == 0 {
                0_u8
            } else {
                let mask = 2 * (((v.coarse_y as u8 & 1) << 1) | (v.coarse_x as u8 & 1));
                let color = ((render.attribute as u8) & (0b0000_0011 << mask)) >> mask;

                (color << 2) | pixel
            };

            //let pixel = 0x3f00 + pixel as usize;
            let pixel = (pixel as usize % 0x0020) + 0x3f00;
            let pixel = unsafe { data.peek_ram(pixel) };

            unsafe { Self::put_dot_on_screen(&mut self.screen, dot, scanline, pixel) }
        }

        // Vertical position.
        if rendering_enabled {
            if copy_vertical_scanline && copy_vertical_dot {
                data.copy_vertical_v();
            }

            if fetch_scanline {
                if self.dot == 256 {
                    data.inc_fine_y();
                } else if self.dot == 257 {
                    data.copy_horizontal_v()
                }
            }
        }

        // Increment the clock, dot and scanline.
        drop(bus);
        self.clock += 1;
        self.dot += 1;

        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;

            if self.scanline == 0 {
                // On odd frames this dot is skipped if rendering is enabled.
                if rendering_enabled && self.frame % 2 == 1 { self.dot += 1 }
            } else if self.scanline == 240 {
                self.frame += 1;
                self.render_sprites();
            } else if self.scanline > 260 {
                trace!("Finished running frame {}.", self.frame);

                self.scanline = -1;

                // Update fps
                let now = Instant::now();
                self.fps = 1_f64 / (now - self.frame_start).as_secs_f64();
                self.frame_start = now;
            }
        }

        // Vblank
        if self.dot == 4 {
            let mut bus = self.bus.borrow_mut();
            match self.scanline {
                -1 => bus.ppu.vblank_clear(),
                241 => bus.start_vblank(),
                _ => {}
            }
        }
    }
}
