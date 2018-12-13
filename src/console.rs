use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

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

    pub render_to_disk: RenderToDisk,

    // PPU information
    pub clock: u32,
    pub frame: u32,
    pub scanline: i32,
    pub cycle: u32,
}

impl Console {
    pub fn new(file: Vec<u8>) -> Self {
        let cartridge = Cartridge::new(file).unwrap();
        let bus = Rc::new(RefCell::new(Bus::with_cartridge(cartridge)));
        let cpu = Cpu::new(bus.clone());

        Self {
            bus,
            cpu,
            render_to_disk: RenderToDisk::Dont,
            clock: 0,
            frame: 0,
            scanline: 0,
            cycle: 0,
        }
    }

    // Logs the current console status for comparing with Nintendulator.
    pub fn log(&self) -> String {
        let reg = &self.cpu.reg;

        format!("{:04X}  {:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:>3} SL:{}",
                reg.get_pc().wrapping_sub(1),
                reg.get_current_instr(),
                reg.get_a(),
                reg.get_x(),
                reg.get_y(),
                reg.get_p().as_u8(),
                reg.get_s(),
                self.cycle,
                self.scanline
        )
    }
    pub fn format_log(log: &str) -> String {
        format!("{} {}", &log[..8], &log[48..])
    }

    // Dismiss a log. Used as callback when the log is not needed.
    pub fn dismiss_log(_: &Self, _: String) -> bool { false }

    // Map the color
    pub fn map_color(dot: u8) -> image::Rgb<u8> {
        #[allow(clippy::match_same_arms)]
        let rgb = match dot {
            0x00 => [84, 84, 84],
            0x01 => [0, 30, 116],
            0x02 => [8, 16, 144],
            0x03 => [48, 0, 136],
            0x04 => [68, 0, 100],
            0x05 => [92, 0, 48],
            0x06 => [84, 4, 0],
            0x07 => [60, 24, 0],
            0x08 => [32, 42, 0],
            0x09 => [8, 58, 0],
            0x0a => [0, 64, 0],
            0x0b => [0, 60, 0],
            0x0c => [0, 50, 60],
            0x0d => [0, 0, 0],

            0x10 => [152, 150, 152],
            0x11 => [8, 76, 196],
            0x12 => [48, 50, 236],
            0x13 => [92, 30, 228],
            0x14 => [136, 20, 176],
            0x15 => [160, 20, 100],
            0x16 => [152, 34, 32],
            0x17 => [120, 60, 0],
            0x18 => [84, 90, 0],
            0x19 => [40, 114, 0],
            0x1a => [8, 124, 0],
            0x1b => [0, 118, 40],
            0x1c => [0, 102, 120],
            0x1d => [0, 0, 0],

            0x20 => [236, 238, 236],
            0x21 => [76, 154, 236],
            0x22 => [120, 124, 236],
            0x23 => [176, 98, 236],
            0x24 => [228, 84, 236],
            0x25 => [236, 88, 180],
            0x26 => [236, 106, 100],
            0x27 => [212, 136, 32],
            0x28 => [160, 170, 0],
            0x29 => [116, 196, 0],
            0x2a => [76, 208, 32],
            0x2b => [56, 204, 108],
            0x2c => [56, 180, 204],
            0x2d => [60, 60, 60],

            0x30 => [236, 238, 236],
            0x31 => [168, 204, 236],
            0x32 => [188, 188, 236],
            0x33 => [212, 178, 236],
            0x34 => [236, 174, 236],
            0x35 => [236, 174, 212],
            0x36 => [236, 180, 176],
            0x37 => [228, 196, 144],
            0x38 => [204, 210, 120],
            0x39 => [180, 222, 120],
            0x3a => [168, 226, 144],
            0x3b => [152, 226, 180],
            0x3c => [160, 214, 228],
            0x3d => [160, 162, 160],

            _ => [0, 0, 0],
        };

        image::Rgb::<u8>(rgb)
    }

    // Render a frame
    pub fn render(&mut self) -> image::RgbImage {
        let mut res = image::RgbImage::new(256, 240);

        let bus = self.bus.borrow_mut();

        let background_table = bus.ppu.base_nametable_addr as usize;
        let attribute_table = background_table + 0x03c0;
        let pattern_table = bus.ppu.background_pattern_table;

        for row in 0..30 {
            for tile in 0..32 {
                let background = background_table + tile + row * 32;
                let attribute = attribute_table + (tile / 2) + (row / 2 * 32);

                let pattern = pattern_table + u16::from(bus.ppu.ram[background]) * 0x10;
                let palette = bus.ppu.ram[attribute];

                // Color mask
                let mask = ((row as u8 & 0x01) << 1) | (tile as u8 & 0x01);
                let color = (palette >> (2 * mask)) << 2;

                // Line of pixels in a pattern
                for (y, line) in (pattern..(pattern + 0x08)).enumerate() {
                    let low = bus.cartridge.read_chr_rom(line);
                    let high = bus.cartridge.read_chr_rom(line + 0x08);

                    let pixels = bits::interlace(low, high);
                    for (x, &pixel) in pixels.iter().enumerate() {
                        let dot = if pixel == 0 { 0_u8 } else { color | pixel };
                        let dot = 0x3f00 + dot as usize;
                        let dot = bus.ppu.ram[dot];

                        let x = 8 * tile as u32 + x as u32;
                        let y = 8 * row as u32 + y as u32;
                        res.put_pixel(x, y, Self::map_color(dot));
                    }
                }
            }
        }

        res
    }

    // Run until some condition is met
    pub fn run_until(&mut self,
                     mut condition: impl FnMut(&Self) -> bool,
                     mut log: impl FnMut(&Self, String) -> bool) {
        loop {
            let mut should_finish = false;

            // Every third PPU cycle, run one cycle of the CPU.
            if self.clock % 3 == 0 {
                self.cpu.step();

                if self.cpu.reg.get_cycle() == cycle::FIRST {
                    should_finish = log(&self, self.log())
                }
            }

            // Increment the clock, cycle and scanline
            self.clock += 1;
            self.cycle += 1;

            if self.cycle > 340 {
                self.cycle = 0;
                self.scanline += 1;

                if self.scanline == 0 {
                    // On odd frames this cycle is skipped if rendering is enabled.
                    let is_rendering = {
                        let bus = self.bus.borrow();
                        bus.ppu.show_background || bus.ppu.show_sprites
                    };
                    if is_rendering && self.frame % 2 == 1 {
                        should_finish = condition(&self);
                        self.cycle += 1
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
                        self.render().save(format!("tests/screenshots/{}", filename)).unwrap();
                    }

                    self.scanline = -1;
                    self.frame += 1;
                }
            }

            // Vblank
            if self.cycle == 0 {
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

    pub fn run_until_cpu_memory_is_not(&mut self,
                                       addr: u16,
                                       data: u8,
                                       log: &mut impl FnMut(&Self, String) -> bool) {
        self.run_until(
            |console| console.bus.borrow_mut().read_cpu(addr) != data,
            log);
    }

    pub fn run_frames(&mut self,
                      frames: u32,
                      log: &mut impl FnMut(&Self, String) -> bool) {
        if frames == 0 { return; }

        let frame = self.frame + frames;
        let scanline = self.scanline;
        let cycle = self.cycle;

        self.run_until(
            |console|
                console.frame == frame
                    && console.scanline == scanline
                    && console.cycle == cycle,
            log);
    }

    pub fn run_log(&mut self, log: &str) {
        let mut log = log.split("\r\n").enumerate();

        self.run_until(|_| false,
                       |console, actual: String| {
                           match log.next() {
                               Some((_, "")) | None => true,

                               Some((line, expected)) => {
                                   assert_eq!(actual, Self::format_log(expected), "\nat line {}\n\n{}",
                                              line, console.cpu);
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
