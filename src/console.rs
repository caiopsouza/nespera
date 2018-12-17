use std::cell::RefCell;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use crate::bus::Bus;
use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::cpu::cycle;
use crate::ppu::Ppu;

pub struct Console {
    pub bus: Rc<RefCell<Bus>>,
    pub cpu: Cpu,
    pub ppu: Ppu,
}

impl Console {
    pub fn new(cartridge: Cartridge) -> Self {
        let bus = Rc::new(RefCell::new(Bus::with_cartridge(cartridge)));
        let cpu = Cpu::new(bus.clone());
        let ppu = Ppu::new(bus.clone());

        Self {
            bus,
            cpu,
            ppu,
        }
    }

    // Logs the current console status for comparing with Nintendulator.
    pub fn log(&self) -> String { self.cpu.log.get() }

    // Dismiss a log. Used as callback when the log is not needed.
    pub fn dismiss_log(_: &Self, _: String) -> bool { false }

    // Run until some condition is met
    pub fn run_until(&mut self,
                     mut condition: impl FnMut(&Self) -> bool,
                     mut log: impl FnMut(&Self, String) -> bool) {
        loop {
            let mut should_finish = false;

            // Every third PPU clock, run one cycle of the CPU.
            if self.ppu.clock % 3 == 0 {
                // Save logs on the first cycle and report it on the last
                match self.cpu.reg.get_cycle() {
                    cycle::FIRST => {
                        let reg = &self.cpu.reg;
                        self.cpu.log.set_reg(reg.clone_with_pc(reg.get_pc() - 1));
                    }

                    cycle::LAST => {
                        let log_res = self.log();
                        if log_res != "" { trace!(target: "opcode", "{}", log_res) }
                        should_finish = log(&self, log_res);

                        self.cpu.log.set_dot(self.ppu.dot);
                        self.cpu.log.set_scanline(self.ppu.scanline);
                    }

                    _ => {}
                }

                self.cpu.step();
            }

            self.ppu.step();

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

        let frame = self.ppu.frame + frames;

        self.run_until(
            |console| console.ppu.frame == frame,
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
