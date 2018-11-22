use std::ops::Generator;
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

                    // NES always read the next byte after an opcode, even if they're not needed.
                    Opcode::Nop(mode::Nop::Implied) => { cycle_implied!(self); }
                    Opcode::Nop(mode::Nop::Immediate) => { cycle_immediate!(self); }
                    Opcode::Nop(mode::Nop::ZeroPage) => { cycle_zero_page!(self); }
                    Opcode::Nop(mode::Nop::ZeroPageX) => { cycle_zero_page_x!(self); }
                    Opcode::Nop(mode::Nop::Absolute) => { cycle_absolute!(self); }
                    Opcode::Nop(mode::Nop::AbsoluteX) => { cycle_absolute_x!(self); }

                    // Load into A
                    Opcode::Lda(mode::Lda::Immediate) => { pipe!(cycle_immediate!(self) => self.cpu.set_a); }
                    Opcode::Lda(mode::Lda::ZeroPage) => { pipe!(cycle_zero_page!(self) => self.cpu.set_a); }
                    Opcode::Lda(mode::Lda::ZeroPageX) => { pipe!(cycle_zero_page_x!(self) => self.cpu.set_a); }
                    Opcode::Lda(mode::Lda::Absolute) => { pipe!(cycle_absolute!(self) => self.cpu.set_a); }
                    Opcode::Lda(mode::Lda::AbsoluteX) => { pipe!(cycle_absolute_x!(self) => self.cpu.set_a); }
                    Opcode::Lda(mode::Lda::AbsoluteY) => { pipe!(cycle_absolute_y!(self) => self.cpu.set_a); }

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

    fn run(bus: Vec<u8>, size: i16, cycles: u32, setup: fn(&mut Nes), result: fn(&mut Nes)) {
        let bus = bus::seq::Bus::new(bus);

        let mut nes = Nes::new(bus);
        setup(&mut nes);

        let mut check = nes.clone();
        result(&mut check);

        check.cycle = cycles.wrapping_add(1);
        check.cpu.pc = nes.cpu.pc.wrapping_add(size as u16).wrapping_add(1);

        nes.run(1);

        assert_eq!(check, nes);
    }

    fn as_is(_: &mut Nes) {}

    #[test]
    fn stp() { run(vec![0x02], 0, 0, as_is, as_is); }

    mod nop {
        use super::*;

        #[test]
        fn implied() { run(vec![0xea], 1, 2, as_is, as_is); }

        #[test]
        fn immediate() { run(vec![0xe2], 2, 2, as_is, as_is); }

        #[test]
        fn zero_page() { run(vec![0x64], 2, 3, as_is, as_is); }

        #[test]
        fn zero_page_x() { run(vec![0x14], 2, 4, as_is, as_is); }

        #[test]
        fn absolute() { run(vec![0x0C], 3, 4, as_is, as_is); }

        #[test]
        fn absolute_x() { run(vec![0x1C, 0xfe, 0x00], 3, 4, as_is, as_is); }

        #[test]
        fn absolute_x_cross_page() {
            run(vec![0x1C, 0xff, 0x00], 3, 5,
                |nes| { nes.cpu.x = 1 },
                as_is,
            );
        }
    }

    mod lda {
        use super::*;

        #[test]
        fn immediate() {
            run(vec![0xA9, 0x01], 2, 2, as_is,
                |nes| { nes.cpu.a = 0x01 },
            );
        }

        #[test]
        fn immediate_zero() {
            run(vec![0xA9, 0x00], 2, 2,
                |nes| { nes.cpu.a = 0x07; },
                |nes| {
                    nes.cpu.set_z();
                    nes.cpu.a = 0x00;
                },
            );
        }

        #[test]
        fn immediate_negative() {
            run(vec![0xA9, -0x01i8 as u8], 2, 2, as_is,
                |nes| {
                    nes.cpu.set_n();
                    nes.cpu.a = -0x01i8 as u8;
                },
            );
        }

        #[test]
        fn zero_page() {
            run(vec![0xA5, 0x02, 0x09], 2, 3, as_is,
                |nes| { nes.cpu.a = 0x09 },
            );
        }

        #[test]
        fn zero_page_x() {
            run(vec![0xB5, 0x02, 0x08, 0x09], 2, 4,
                |nes| { nes.cpu.x = 0x01 },
                |nes| { nes.cpu.a = 0x09 },
            );
        }
    }
}
