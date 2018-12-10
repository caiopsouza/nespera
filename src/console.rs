use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::mapper::Cartridge;

pub struct Console {
    pub bus: Rc<RefCell<Bus>>,
    pub cpu: Cpu,

    // PPU information
    pub scanline: u32,
    pub dot: u32,
}

impl Console {
    pub fn new(file: Vec<u8>) -> Self {
        let cartridge = Cartridge::new(file).unwrap();
        let bus = Rc::new(RefCell::new(Bus::with_cartridge(cartridge)));
        let cpu = Cpu::new(bus.clone());

        Self { bus, cpu, scanline: 0, dot: 0 }
    }

    pub fn run_until(&mut self, condition: impl Fn(&Console) -> bool) {
        loop {
            // Vblank
            if self.dot == 1 {
                if self.scanline == 241 {
                    self.bus.borrow_mut().start_vblank()
                } else if self.scanline == 261 {
                    self.bus.borrow_mut().ppu.vblank_clear()
                }
            }

            // Every third PPU cycle, run one cycle of the CPU.
            // PPU is not emulated yet.
            if (self.scanline * 261 + self.dot) % 3 == 0 {
                self.cpu.step();
                if condition(&self) { return; }
            }

            // Increment the dot and scanline
            self.dot += 1;

            if self.dot > 240 {
                self.dot = 0;
                self.scanline += 1;

                if self.scanline > 340 { self.scanline = 0 }
            }
        }
    }

    pub fn run_until_memory_is(&mut self, addr: u16, data: u8) {
        self.run_until(|console| console.bus.borrow_mut().read(addr) == data);
    }

    pub fn run_frames(&mut self, frames: u32) {
        if frames == 0 { return; }

        let scanline = self.scanline;
        let dot = (if self.dot == 0 { 240 } else { self.dot }) - 3;

        for _ in 0..frames {
            self.run_until(|console| console.scanline == scanline && console.dot == dot);
        }
    }
}
