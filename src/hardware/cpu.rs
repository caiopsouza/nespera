use hardware::flags::Flags;

use std::num::Wrapping;
use std::fmt;

// Power up state of the CPU
const PC_POWER_UP_STATE: u16 = 0;
const SP_POWER_UP_STATE: u8 = 0xfd;

#[derive(Copy, Clone)]
pub struct Cpu {
    pub a: u8 /* Accumulator*/,
    pub x: u8 /* Index X*/,
    pub y: u8 /* Index Y*/,
    pub p: Flags /* Flags*/,
    pub pc: u16 /* Program counter*/,
    pub sp: u8 /* Stack pointer*/,
}

impl fmt::Debug for Cpu {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter,
               "a: {:02x}, x: {:02x}, y: {:02x}, pc: {:04x}, sp: {:02x}, p: {:02x} {}{}{}{}{}{}{}{}",
               self.a, self.x, self.y, self.pc, self.sp, self.p,
               if self.get_n() { 'n' } else { '_' },
               if self.get_v() { 'v' } else { '_' },
               if self.get_u() { 'u' } else { '_' },
               if self.get_b() { 'b' } else { '_' },
               if self.get_d() { 'd' } else { '_' },
               if self.get_i() { 'i' } else { '_' },
               if self.get_z() { 'z' } else { '_' },
               if self.get_c() { 'c' } else { '_' })
    }
}

impl Cpu {
    // Create a new CPU. It has the default "power up" state.
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: PC_POWER_UP_STATE,
            sp: SP_POWER_UP_STATE,
            p: Flags::InterruptDisable | Flags::Unused,
        }
    }

    pub fn new_from_pc(pc: u16) -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc,
            sp: SP_POWER_UP_STATE,
            p: Flags::InterruptDisable | Flags::Unused,
        }
    }

    // Getters for registers
    pub fn get_a(&self) -> u8 { self.a }
    pub fn get_x(&self) -> u8 { self.x }
    pub fn get_y(&self) -> u8 { self.y }
    pub fn get_p(&self) -> u8 { self.p.bits() }
    pub fn get_sp(&self) -> u8 { self.sp }
    pub fn get_pc(&self) -> u16 { self.pc }

    // Getters for flags
    pub fn get_c(&self) -> bool { self.p.intersects(Flags::Carry) }
    pub fn get_z(&self) -> bool { self.p.intersects(Flags::Zero) }
    pub fn get_i(&self) -> bool { self.p.intersects(Flags::InterruptDisable) }
    pub fn get_d(&self) -> bool { self.p.intersects(Flags::DecimalMode) }
    pub fn get_b(&self) -> bool { self.p.intersects(Flags::BreakCommand) }
    pub fn get_u(&self) -> bool { self.p.intersects(Flags::Unused) }
    pub fn get_v(&self) -> bool { self.p.intersects(Flags::Overflow) }
    pub fn get_n(&self) -> bool { self.p.intersects(Flags::Negative) }

    // Reset PC
    pub fn reset_pc(&mut self) { self.pc = PC_POWER_UP_STATE; }

    // Increments PC
    pub fn inc_pc(&mut self) { self.pc = (Wrapping(self.pc) + Wrapping(1)).0; }

    // Set registers
    pub fn set_a(&mut self, value: u8) {
        self.a = value;
        self.p.change_zero_and_negative(value);
    }

    pub fn set_x(&mut self, value: u8) {
        self.x = value;
        self.p.change_zero_and_negative(value);
    }

    pub fn set_y(&mut self, value: u8) {
        self.y = value;
        self.p.change_zero_and_negative(value);
    }

    pub fn set_p(&mut self, value: u8) { self.p = value.into(); }

    pub fn set_pc(&mut self, value: u16) { self.pc = value; }

    pub fn set_sp(&mut self, value: u8) { self.sp = value; }

    // Adds a value into A
    pub fn adc_a(&mut self, value: u8) {
        let res = self.a as u16 + value as u16 + self.get_c() as u16;

        self.p.change_zero_and_negative(res as u8);

        // When adding, carry happens if bit 8 is set
        self.p.set(Flags::Carry, (res & 0x0100u16) != 0);

        // Overflow happens when the sign of the addends is the same and differs from the sign of the sum
        self.p.set(Flags::Overflow, (!(self.a ^ value) & (self.a ^ res as u8) & 0x80) != 0);

        // Save the result
        self.a = res as u8;
    }

    // Subtracts a value into A
    pub fn sbc_a(&mut self, value: u8) {
        // Since you should subtract (1 - carry) inverting the value
        // has the same effect as a two's complement after the carry is added
        // Carry is inverted.
        self.adc_a(!value);
    }

    // Comparisons
    pub fn cmp_a(&mut self, value: u8) { self.p.change_cmp(self.a, value); }
    pub fn cmp_x(&mut self, value: u8) { self.p.change_cmp(self.x, value); }
    pub fn cmp_y(&mut self, value: u8) { self.p.change_cmp(self.y, value); }

    // Shifts A left
    pub fn left_shift_a(&mut self) {
        self.p.change_left_shift(self.a);
        let a = self.a << 1;
        self.set_a(a);
    }

    // Shifts A right
    pub fn right_shift_a(&mut self) {
        self.p.change_right_shift(self.a);
        let a = self.a >> 1;
        self.set_a(a);
    }

    // Rotates A left
    pub fn left_rotate_a(&mut self) {
        let p = self.p;

        // Same as left shift
        self.p.change_left_shift(self.a);

        let a = (self.a << 1) | (p.contains(Flags::Carry) as u8);
        self.set_a(a);
    }

    // Rotates A right
    pub fn rotate_a_right(&mut self) {
        let p = self.p;
        self.p.change_right_rotate(self.a);
        let a = (self.a >> 1) | ((p.contains(Flags::Carry) as u8) << 7);
        self.set_a(a);
    }

    // Flag operations
    pub fn set_flag(&mut self, flag: Flags) { self.p.insert(flag); }
    pub fn clear_flag(&mut self, flag: Flags) { self.p.remove(flag); }
}
