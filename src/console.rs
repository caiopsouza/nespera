use std::cell::RefCell;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::time::Instant;

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::cpu::cycle;
use crate::utils::bits;

#[derive(PartialOrd, PartialEq, Ord, Eq)]
pub enum RenderToDisk {
    Dont,
    Frame,
    Filename(String),
}

pub struct Console {
    pub bus: Rc<RefCell<Bus>>,
    pub cpu: Cpu,

    // Fps calculation
    frame_start: Instant,
    pub fps: f64,

    // Rendering
    pub screen: image::RgbaImage,
    pub render_to_disk: RenderToDisk,

    // PPU information
    pub clock: u32,
    pub frame: u32,
    pub scanline: i32,
    pub dot: u32,
}

impl Console {
    pub fn new(file: Vec<u8>) -> Self {
        let cartridge = Cartridge::new(file).unwrap();
        let bus = Rc::new(RefCell::new(Bus::with_cartridge(cartridge)));
        let cpu = Cpu::new(bus.clone());

        Self {
            bus,
            cpu,
            frame_start: Instant::now(),
            fps: 0_f64,
            screen: image::RgbaImage::new(256, 240),
            render_to_disk: RenderToDisk::Dont,
            clock: 0,
            frame: 0,
            scanline: 0,
            dot: 0,
        }
    }

    // Logs the current console status for comparing with Nintendulator.
    pub fn log(&self) -> String { self.cpu.log.get() }

    // Dismiss a log. Used as callback when the log is not needed.
    pub fn dismiss_log(_: &Self, _: String) -> bool { false }

    // Map the color
    pub fn map_color(dot: u8) -> image::Rgba<u8> {
        #[allow(clippy::match_same_arms)]
        let rgba = match dot {
            0x00 => [84, 84, 84, 255],
            0x01 => [0, 30, 116, 255],
            0x02 => [8, 16, 144, 255],
            0x03 => [48, 0, 136, 255],
            0x04 => [68, 0, 100, 255],
            0x05 => [92, 0, 48, 255],
            0x06 => [84, 4, 0, 255],
            0x07 => [60, 24, 0, 255],
            0x08 => [32, 42, 0, 255],
            0x09 => [8, 58, 0, 255],
            0x0a => [0, 64, 0, 255],
            0x0b => [0, 60, 0, 255],
            0x0c => [0, 50, 60, 255],
            0x0d => [0, 0, 0, 255],
            0x0e => [0, 0, 0, 255],
            0x0f => [0, 0, 0, 255],

            0x10 => [152, 150, 152, 255],
            0x11 => [8, 76, 196, 255],
            0x12 => [48, 50, 236, 255],
            0x13 => [92, 30, 228, 255],
            0x14 => [136, 20, 176, 255],
            0x15 => [160, 20, 100, 255],
            0x16 => [152, 34, 32, 255],
            0x17 => [120, 60, 0, 255],
            0x18 => [84, 90, 0, 255],
            0x19 => [40, 114, 0, 255],
            0x1a => [8, 124, 0, 255],
            0x1b => [0, 118, 40, 255],
            0x1c => [0, 102, 120, 255],
            0x1d => [0, 0, 0, 255],
            0x1e => [0, 0, 0, 255],
            0x1f => [0, 0, 0, 255],

            0x20 => [236, 238, 236, 255],
            0x21 => [76, 154, 236, 255],
            0x22 => [120, 124, 236, 255],
            0x23 => [176, 98, 236, 255],
            0x24 => [228, 84, 236, 255],
            0x25 => [236, 88, 180, 255],
            0x26 => [236, 106, 100, 255],
            0x27 => [212, 136, 32, 255],
            0x28 => [160, 170, 0, 255],
            0x29 => [116, 196, 0, 255],
            0x2a => [76, 208, 32, 255],
            0x2b => [56, 204, 108, 255],
            0x2c => [56, 180, 204, 255],
            0x2d => [60, 60, 60, 255],
            0x2e => [0, 0, 0, 255],
            0x2f => [0, 0, 0, 255],

            0x30 => [236, 238, 236, 255],
            0x31 => [168, 204, 236, 255],
            0x32 => [188, 188, 236, 255],
            0x33 => [212, 178, 236, 255],
            0x34 => [236, 174, 236, 255],
            0x35 => [236, 174, 212, 255],
            0x36 => [236, 180, 176, 255],
            0x37 => [228, 196, 144, 255],
            0x38 => [204, 210, 120, 255],
            0x39 => [180, 222, 120, 255],
            0x3a => [168, 226, 144, 255],
            0x3b => [152, 226, 180, 255],
            0x3c => [160, 214, 228, 255],
            0x3d => [160, 162, 160, 255],
            0x3e => [0, 0, 0, 255],
            0x3f => [0, 0, 0, 255],

            _ => {
                error!("Indexing color not mapped: 0x{:02x}. Defaulting to black.", dot);
                [0, 0, 0, 255]
            }
        };

        image::Rgba::<u8>(rgba)
    }

    // Calculates a matrix index
    fn index(base: usize, x: usize, y: usize, width: usize) -> usize {
        base + x + y * width
    }

    // Render the current dot
    pub fn render(&mut self) {
        // Not visible in these cases.
        if !(0..240_i32).contains(&self.scanline) { return; }
        if !(0..256_u32).contains(&self.dot) { return; }

        let bus = self.bus.borrow_mut();

        // Tables
        let background_table = bus.ppu.base_nametable_addr as usize;
        let attribute_table = background_table + 0x03c0;
        let pattern_table = bus.ppu.background_pattern_table;

        // Position on the name table.
        let row = self.scanline as usize / 8;
        let tile = self.dot as usize / 8;

        // Background
        let background = Self::index(background_table, tile, row, 32);
        let pattern = pattern_table + u16::from(bus.ppu.ram[background]) * 0x10;

        // Palette
        let palette_addr = Self::index(attribute_table, tile / 4, row / 4, 8);
        let palette = bus.ppu.ram[palette_addr];

        // Position inside the pattern.
        let x = self.dot as u8 % 8;
        let y = self.scanline as u16 % 8;

        // Pixel
        let low = bus.cartridge.read_chr_rom(pattern + y);
        let high = bus.cartridge.read_chr_rom(pattern + y + 8);
        let pixel = bits::interlace(low, high, x);

        // Color
        let mask = 2 * (((row as u8 & 1) << 1) | (tile as u8 & 1));
        let color = (palette & (0b0000_0011 << mask)) >> mask;

        // Dot
        let dot = if pixel == 0 { 0_u8 } else { (color << 2) | pixel };
        let dot = 0x3f00 + dot as usize;
        let dot = bus.ppu.ram[dot];
        self.screen.put_pixel(self.dot, self.scanline as u32, Self::map_color(dot));
    }

    // Run until some condition is met
    pub fn run_until(&mut self,
                     mut condition: impl FnMut(&Self) -> bool,
                     mut log: impl FnMut(&Self, String) -> bool) {
        loop {
            let mut should_finish = false;

            // Every third PPU dot, run one cycle of the CPU.
            if self.clock % 3 == 0 {
                // Save logs on the first cycle and report it on the last
                match self.cpu.reg.get_cycle() {
                    cycle::FIRST => {
                        let reg = &self.cpu.reg;
                        self.cpu.log.set_reg(reg.clone_with_pc(reg.get_pc() - 1));
                    }

                    cycle::LAST => {
                        let log_res = self.log();
                        if log_res != "" {
                            trace!(target: "opcode", "{}", log_res);
                            should_finish = log(&self, log_res)
                        }

                        self.cpu.log.set_dot(self.dot);
                        self.cpu.log.set_scanline(self.scanline);
                    }

                    _ => {}
                }

                self.cpu.step();
            }

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
                    if is_rendering && self.frame % 2 == 1 {
                        should_finish = condition(&self);
                        self.dot += 1
                    }
                } else if self.scanline > 260 {
                    trace!("Finished running frame {}.", self.frame);

                    // Render image to disk
                    if self.render_to_disk != RenderToDisk::Dont {
                        let filename = match self.render_to_disk {
                            RenderToDisk::Frame => { format!("{:06}.png", self.frame) }
                            RenderToDisk::Filename(ref filename) => { filename.clone() }
                            RenderToDisk::Dont => { unimplemented!() }
                        };
                        self.screen.save(format!("tests/screenshots/{}", filename)).unwrap();
                    }

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

            if should_finish || condition(&self) { break; }
        }
    }

    pub fn run_until_cpu_memory_is(&mut self,
                                   addr: u16,
                                   data: u8,
                                   log: &mut impl FnMut(&Self, String) -> bool) {
        self.run_until(
            |console| console.bus.borrow_mut().read_cpu(addr) == data,
            log);
    }

    pub fn run_until_cpu_memory_is_not(&mut self, addr: u16, data: u8) {
        self.run_until(
            |console| console.bus.borrow_mut().read_cpu(addr) != data,
            Self::dismiss_log);
    }

    pub fn run_frames(&mut self, frames: u32) {
        if frames == 0 { return; }

        let frame = self.frame + frames;
        let scanline = self.scanline;
        let dot = self.dot;

        self.run_until(
            |console|
                console.frame == frame
                    && console.scanline == scanline
                    && console.dot == dot,
            Self::dismiss_log);
    }

    pub fn run_log(&mut self, log: &str) {
        let mut log_file = File::open(log).unwrap();
        let mut log = String::new();
        log_file.read_to_string(&mut log).unwrap();
        let mut log = log.split("\r\n").enumerate();

        self.run_until(|_| false,
                       |console, actual: String| {
                           match log.next() {
                               Some((_, "")) | None => true,

                               Some((line, expected)) => {
                                   assert_eq!(actual, expected, "\nat line {}\n\n{}",
                                              line + 1, console.cpu);
                                   false
                               }
                           }
                       });
    }
}

impl fmt::Debug for Console {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        writeln!(formatter, "{:?}\n", self.cpu)?;
        write!(formatter, "{:?}", self.bus)
    }
}
