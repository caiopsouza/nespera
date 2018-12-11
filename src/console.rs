use std::cell::RefCell;
use std::rc::Rc;

use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cpu::cycle;
use crate::mapper::Cartridge;

pub struct Console {
    pub bus: Rc<RefCell<Bus>>,
    pub cpu: Cpu,

    instr_pc: u16,

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
        let instr_pc = cpu.reg.get_pc();

        Self { bus, cpu, instr_pc, clock: 0, frame: 1, scanline: 0, cycle: 0 }
    }

    // Logs the current console status for comparing with Nintendulator.
    pub fn log(&self) -> String {
        format!("{:04X}  {:02X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{:>3} SL:{}",
                self.instr_pc,
                self.cpu.reg.get_current_instr(),
                self.cpu.reg.get_a(),
                self.cpu.reg.get_x(),
                self.cpu.reg.get_y(),
                self.cpu.reg.get_p().as_u8(),
                self.cpu.reg.get_s(),
                self.cycle,
                self.scanline
        )
    }

    pub fn run_until(&mut self,
                     condition: impl Fn(&Console) -> bool,
                     log: &mut impl FnMut(&Console, String)) {
        loop {
            // Every third PPU cycle, run one cycle of the CPU.
            let mut should_finish = false;
            if self.clock % 3 == 0 {
                self.cpu.step();

                match self.cpu.reg.get_cycle() {
                    cycle::FIRST => log(&self, self.log()),
                    cycle::LAST => self.instr_pc = self.cpu.reg.get_pc(),
                    _ => {}
                }

                should_finish = condition(&self);
            }

            // Increment the clock, cycle and scanline
            self.clock += 1;
            self.cycle += 1;

            if self.cycle > 340 {
                self.cycle = 0;
                self.scanline += 1;

                if self.scanline == 0 {
                    // On odd frames this cycle is skipped if rendering is enabled.
                    let bus = self.bus.borrow();
                    if self.frame % 2 == 1 && (bus.ppu.show_background || bus.ppu.show_sprites) {
                        self.cycle += 1
                    }
                } else if self.scanline > 260 {
                    self.scanline = -1;
                    self.frame += 1;

                    trace!("Finished running a frame.");
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

            if should_finish { break; }
        }
    }

    pub fn run_until_memory_is(&mut self, addr: u16, data: u8, log: &mut impl FnMut(&Console, String)) {
        self.run_until(
            |console| console.bus.borrow_mut().read(addr) == data,
            log);
    }

    pub fn run_until_memory_is_not(&mut self, addr: u16, data: u8, log: &mut impl FnMut(&Console, String)) {
        self.run_until(
            |console| console.bus.borrow_mut().read(addr) != data,
            log);
    }

    pub fn run_frames(&mut self, frames: u32, log: &mut impl FnMut(&Console, String)) {
        if frames == 0 { return; }

        let scanline = self.scanline;
        let cycle = self.cycle;

        for _ in 0..frames {
            self.run_until(
                |console| console.scanline == scanline && console.cycle == cycle,
                log);
        }
    }
}
