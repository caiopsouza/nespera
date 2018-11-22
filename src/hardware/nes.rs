use std::ops::Generator;

use hardware::bus::Bus;
use hardware::cpu::Cpu;
use hardware::flags;
use hardware::opc;
use hardware::opc::mode;
use hardware::opc::Opcode;
use std::ops::GeneratorState;

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
        // Helpers to yield.
        macro_rules! cycle { () => {{ self.cycle += 1; yield false; }; }}
        macro_rules! cycle_fecth {
            () => {{
                self.cpu.inc_pc();
                let next = self.bus.read(self.cpu.pc);
                cycle!();
                next
            }};
        }

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
                            mode::Nop::Implicit => { cycle!(); }
                            mode::Nop::Accumulator | mode::Nop::Immediate | mode::Nop::ZeroPage => {
                                cycle_fecth!();
                                cycle!();
                            }
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

    fn test(bus: Vec<u8>, cycles: u32, pc_offset: i16, checker: fn(Nes) -> Nes) {
        let bus = bus::seq::Bus::new(bus);

        let mut nes = Nes::new(bus);
        let mut check = nes.clone();

        check.cycle = cycles.wrapping_add(1);
        check.cpu.pc = nes.cpu.pc.wrapping_add(pc_offset as u16).wrapping_add(1);

        check = checker(check);

        nes.run(1);

        assert!(check == nes, r#"assertion failed: `(expected == encountered)`
    expected: `{:02X?}`,
 encountered: `{:02X?}`"#, check, nes);
    }

    #[test]
    fn stp() { test(vec![0x02], 0, 0, |nes| nes); }

    #[cfg(test)]
    mod nop {
        use super::*;

        #[test]
        fn implicit() { test(vec![0xea], 2, 1, |nes| nes); }

        #[test]
        fn accumulator() { test(vec![0x0c], 3, 2, |nes| nes); }

        #[test]
        fn immediate() { test(vec![0xe2], 3, 2, |nes| nes); }

        #[test]
        fn zero_page() { test(vec![0x64], 3, 2, |nes| nes); }
    }
}
