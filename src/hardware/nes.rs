use std::ops::Generator;

use hardware::bus::Bus;
use hardware::cpu::Cpu;
use hardware::flags;
use hardware::opc;

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

    // Create a function to step the emulator
    pub fn step(&mut self) -> impl Generator<Yield=(), Return=()> + '_ {
        // Helpers to yield. Also increments cycle
        macro_rules! cycle { () => {{ self.cycle += 1; yield; }}; }

        // Run a single step of the CPU
        move || {
            loop {
                // Fetch the opcode to execute
                let opcode = self.bus.read(self.cpu.pc);
                self.cpu.inc_pc();
                cycle!();

                match opc::OPCODES[opcode as usize] {
                    // STP crashes the CPU. I'll just return for now.
                    opc::STP => { return; }

                    // Does nothing
                    opc::NOP => { cycle!(); }
                    opc::NOP_IMM => {
                        cycle!();
                        cycle!();
                    }

                    // Opcodes not implemented
                    opc::NONE => panic!("Opcode not set: {:#X}", opcode),
                    opc::Opcode { .. } => panic!("Opcode not implemented: {:#X}", opcode)
                }
            }
        }
    }

    // Run the NES by the amount of cycles specified
    pub fn run(&mut self, cycles: u32) {
        let mut step = self.step();
        for _ in 0..cycles {
            unsafe {
                step.resume();
            }
        }
    }
}

#[cfg(test)]
mod opcodes {
    type Nes = super::Nes<bus::seq::Bus>;

    use hardware::bus;

    fn test(bus: Vec<u8>, checker: fn(Nes) -> Nes) {
        let bus = bus::seq::Bus::new(bus);
        let mut nes = Nes::new(bus);
        let check = checker(nes.clone());
        nes.run(2);
        assert_eq!(check, nes);
    }

    #[test]
    #[should_panic]
    fn stp() { test(vec![0x02], |nes| nes); }

    #[test]
    fn nop() { test(vec![0xea], |nes| Nes { cycle: 2, ..nes }); }
}
