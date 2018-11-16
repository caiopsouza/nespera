pub use flags::Flags;

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

    // Getters for registers
    pub fn get_a(&self) -> u8 { self.a }
    pub fn get_x(&self) -> u8 { self.x }
    pub fn get_y(&self) -> u8 { self.y }
    pub fn get_p(&self) -> u8 { self.p.bits() }
    pub fn get_pc(&self) -> u16 { self.pc }
    pub fn get_sp(&self) -> u8 { self.sp }

    // Getters for flags
    pub fn get_c(&self) -> bool { self.p.intersects(Flags::Carry) }
    pub fn get_z(&self) -> bool { self.p.intersects(Flags::Zero) }
    pub fn get_i(&self) -> bool { self.p.intersects(Flags::InterruptDisable) }
    pub fn get_d(&self) -> bool { self.p.intersects(Flags::DecimalMode) }
    pub fn get_b(&self) -> bool { self.p.intersects(Flags::BreakCommand) }
    pub fn get_u(&self) -> bool { self.p.intersects(Flags::Unused) }
    pub fn get_o(&self) -> bool { self.p.intersects(Flags::Overflow) }
    pub fn get_n(&self) -> bool { self.p.intersects(Flags::Negative) }

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

    // Comparisons
    pub fn cmp_a(&mut self, value: u8) { self.p.znc_cmp(self.a, value); }
    pub fn cmp_x(&mut self, value: u8) { self.p.znc_cmp(self.x, value); }
    pub fn cmp_y(&mut self, value: u8) { self.p.znc_cmp(self.y, value); }

    // Shifts A left
    pub fn shift_a_left(&mut self) {
        self.p.znc_left_shift(self.a);
        self.a <<= 1;
    }

    // Shifts A right
    pub fn shift_a_right(&mut self) {
        self.p.znc_right_shift(self.a);
        self.a >>= 1;
    }

    // Rotates A left
    pub fn rotate_a_left(&mut self) {
        self.p.znc_left_shift(self.a);
        self.a = (self.a << 1) | (self.p.bits() & Flags::Carry.bits());
    }

    // Rotates A right
    pub fn rotate_a_right(&mut self) {
        self.p.znc_right_rotate(self.a);
        self.a = (self.a >> 1) | ((self.p.bits() & Flags::Carry.bits()) << 7);
    }

    // Flag operations
    pub fn set_flag(&mut self, flag: Flags) { self.p.insert(flag); }
    pub fn clear_flag(&mut self, flag: Flags) { self.p.remove(flag); }
}
