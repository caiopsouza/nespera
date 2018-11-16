use flags::Flags;
use std::num::Wrapping;

// Power up state of the CPU
const PC_POWER_UP_STATE: u16 = 0;
const SP_POWER_UP_STATE: u8 = 0xfd;

#[derive(Debug, Copy, Clone)]
pub struct Cpu {
    pub a: u8 /* Accumulator*/,
    pub x: u8 /* Index X*/,
    pub y: u8 /* Index Y*/,
    pub p: Flags /* Flags*/,
    pub pc: u16 /* Program counter*/,
    pub sp: u8 /* Stack pointer*/,
}

impl Cpu {
    // Create a new CPU. It has the default "power up" state.
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: PC_POWER_UP_STATE,
            sp: SP_POWER_UP_STATE,
            p: Flags::InterruptDisable | Flags::Unused | Flags::BreakCommand,
        }
    }

    // Reset PC
    pub fn reset_pc(&mut self) { self.pc = PC_POWER_UP_STATE; }

    // Increments PC by the value supplied
    pub fn inc_pc(&mut self) {
        self.pc = (Wrapping(self.pc) + Wrapping(1)).0;
    }

    // Set registers
    pub fn set_a(&mut self, value: u8) {
        self.a = value;
        self.p.zn(value);
    }

    pub fn set_x(&mut self, value: u8) {
        self.x = value;
        self.p.zn(value);
    }

    pub fn set_y(&mut self, value: u8) {
        self.y = value;
        self.p.zn(value);
    }

    pub fn set_p(&mut self, value: u8) { self.p = value.into(); }

    pub fn set_sp(&mut self, value: u8) { self.sp = value; }

    // Sums a value into A
    pub fn adc_a(&mut self, value: u8) {
        self.p.znco_adc(self.a, value);
        self.a = (Wrapping(self.a) + Wrapping(value)).0;
    }

    // Sums a value into A
    pub fn sbc_a(&mut self, value: u8) {
        self.p.znco_sbc(self.a, value);
        self.a = (Wrapping(self.a) - Wrapping(value)).0;
    }
}
