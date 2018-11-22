use std::ops::Generator;
#[cfg(test)]
use std::ops::GeneratorState;

use hardware::bus::Bus;
use hardware::cpu::Cpu;
use hardware::flags;
use hardware::opc;
use hardware::opc::mode;
use hardware::opc::Opcode;

#[derive(Debug, PartialOrd, PartialEq, Clone)]
pub struct Nes<TBus: Bus> {
    // Current cycle
    pub cycle: u32,

    // Cpu
    pub cpu: Cpu,

    // Bus
    pub bus: TBus,
}

impl<TBus: Bus> Nes<TBus> {
    pub fn new(bus: TBus) -> Self {
        let mut res = Self {
            cycle: 0,
            cpu: Cpu::new(),
            bus,
        };

        res.cpu.pc = 0xc000;
        res.cpu.p = flags::INTERRUPT_DISABLE | flags::UNUSED;

        res
    }

    // Create a function to step the emulator. Yields true if an opcode has finished.
    pub fn step(&mut self) -> impl Generator<Yield=(bool), Return=()> + '_ {

        // Run a single step of the CPU
        let mut stepper = move || {
            loop {
                // Fetch the opcode to execute
                let opcode = self.bus.read(self.cpu.pc);
                self.cpu.inc_pc();
                self.cycle += 1;
                yield true;

                match &opc::OPCODES[opcode as usize] {
                    // STP prevent cycle to advance effectively crashing the CPU.
                    Opcode::Stp => { loop { yield (true); } }

                    // Does nothing
                    Opcode::Nop(mode) => {
                        match mode {
                            // NES always read the next byte after an opcode, even if they're not needed.
                            mode::Nop::Implied => { cycle_read!(self, self.cpu.pc); }
                            mode::Nop::Immediate => { cycle_fetch!(self); }
                            mode::Nop::ZeroPage => { cycle_zero_page!(self); }
                            mode::Nop::ZeroPageX => { cycle_zero_page_x!(self); }
                            mode::Nop::Absolute => { cycle_absolute!(self); }
                            mode::Nop::AbsoluteX => { cycle_absolute_x!(self); }
                        }
                    }

                    // Not implemented
                    Opcode::None => panic!("Opcode not implemented: 0x{:02X}", opcode)
                }
            }
        };

        // Fetch the first opcode.
        // Every instruction finishes fetching the next opcode, so this is the "zeroth" instruction.
        unsafe { stepper.resume(); }
        stepper
    }

    // Run for the specified amount of instructions.
    #[cfg(test)]
    pub fn run(&mut self, mut instr_amount: u32) {
        let mut step = self.step();

        while instr_amount > 0 {
            unsafe {
                match step.resume() {
                    GeneratorState::Yielded(false) => {}
                    GeneratorState::Yielded(true) | GeneratorState::Complete(_) => { instr_amount -= 1 }
                }
            }
        }
    }
}

#[cfg(test)]
mod opcodes {
    use hardware::bus;

    type Nes = super::Nes<bus::seq::Bus>;

    fn run(bus: Vec<u8>, size: i16, cycles: u32, setup: fn(&mut Nes)) {
        let bus = bus::seq::Bus::new(bus);

        let mut nes = Nes::new(bus);
        setup(&mut nes);

        let mut check = nes.clone();

        check.cycle = cycles.wrapping_add(1);
        check.cpu.pc = nes.cpu.pc.wrapping_add(size as u16).wrapping_add(1);

        nes.run(1);

        assert!(check == nes, r#"assertion failed: `(expected == encountered)`
    expected: `{:02X?}`,
 encountered: `{:02X?}`"#, check, nes);
    }

    #[test]
    fn stp() { run(vec![0x02], 0, 0, |_nes| {}); }

    #[cfg(test)]
    mod nop {
        use super::*;

        #[test]
        fn implied() { run(vec![0xea], 1, 2, |_nes| {}); }

        #[test]
        fn immediate() { run(vec![0xe2], 2, 2, |_nes| {}); }

        #[test]
        fn zero_page() { run(vec![0x64], 2, 3, |_nes| {}); }

        #[test]
        fn zero_page_x() { run(vec![0x14], 2, 4, |_nes| {}); }

        #[test]
        fn absolute() { run(vec![0x0C], 3, 4, |_nes| {}); }

        #[test]
        fn absolute_x() { run(vec![0x1C, 0xfe, 0x00], 3, 4, |_nes| {}); }

        #[test]
        fn absolute_x_cross_page() {
            run(vec![0x1C, 0xff, 0x00], 3, 5,
                |nes| { nes.cpu.x = 1 },
            );
        }
    }
}
