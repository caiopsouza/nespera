use hardware::flags;
use std::fmt;

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Cpu {
    // Accumulator
    pub a: u8,

    // Index X
    pub x: u8,

    // Index Y
    pub y: u8,

    // Flags
    pub p: u8,

    // Program counter
    pub pc: u16,

    // Stack pointer
    pub s: u8,
}

impl Cpu {
    // Create a new CPU. It has the default "power up" state.
    pub fn new() -> Self { Self { a: 0, x: 0, y: 0, pc: 0, s: 0, p: 0 } }

    // Increment PC
    pub fn inc_pc(&mut self) { self.pc = self.pc.wrapping_add(1); }

    // Getters for the P Flag
    pub fn get_c(&self) -> bool { (self.p & flags::CARRY) != 0 }
    pub fn get_z(&self) -> bool { (self.p & flags::ZERO) != 0 }
    pub fn get_i(&self) -> bool { (self.p & flags::INTERRUPT_DISABLE) != 0 }
    pub fn get_d(&self) -> bool { (self.p & flags::DECIMAL_MODE) != 0 }
    pub fn get_b(&self) -> bool { (self.p & flags::BREAK_COMMAND) != 0 }
    pub fn get_u(&self) -> bool { (self.p & flags::UNUSED) != 0 }
    pub fn get_v(&self) -> bool { (self.p & flags::OVERFLOW) != 0 }
    pub fn get_n(&self) -> bool { (self.p & flags::NEGATIVE) != 0 }

    // Setters for the P Flag
    pub fn set_c(&mut self) { self.p |= flags::CARRY; }
    pub fn set_z(&mut self) { self.p |= flags::ZERO; }
    pub fn set_i(&mut self) { self.p |= flags::INTERRUPT_DISABLE; }
    pub fn set_d(&mut self) { self.p |= flags::DECIMAL_MODE; }
    pub fn set_b(&mut self) { self.p |= flags::BREAK_COMMAND; }
    pub fn set_u(&mut self) { self.p |= flags::UNUSED; }
    pub fn set_v(&mut self) { self.p |= flags::OVERFLOW; }
    pub fn set_n(&mut self) { self.p |= flags::NEGATIVE; }

    // Change the value of a flag
    pub fn change_flag(&mut self, flags: u8, condition: bool) {
        if condition { self.p |= flags } else { self.p &= !flags }
    }

    pub fn change_flag_zero(&mut self, value: u8) {
        self.change_flag(flags::ZERO, value == 0);
    }

    pub fn change_flag_negative(&mut self, value: u8) {
        // Flag Negative is the 7th bit and should be set when the 7th bit of the value is set.
        // Just copy it, then.
        self.p = (self.p & !flags::NEGATIVE) | (value & flags::NEGATIVE);
    }

    // Setters for the register
    pub fn set_a(&mut self, value: u8) {
        self.change_flag_zero(value);
        self.change_flag_negative(value);
        self.a = value;
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter,
               "a: {:02X}, x: {:02X}, y: {:02X}, pc: {:04X}, s: {:02X}, p: {:02X} {}{}{}{}{}{}{}{}",
               self.a, self.x, self.y, self.pc, self.s, self.p,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_zero() {
        let mut cpu = Cpu::new();
        cpu.p = !flags::ZERO;
        assert!(!cpu.get_z());
        cpu.change_flag_zero(0);
        assert!(cpu.get_z());
    }

    #[test]
    fn flag_not_zero() {
        let mut cpu = Cpu::new();
        cpu.p = flags::ZERO;
        assert!(cpu.get_z());
        cpu.change_flag_zero(0x10);
        assert!(!cpu.get_z());
    }

    #[test]
    fn flag_negative() {
        let mut cpu = Cpu::new();
        cpu.p = !flags::NEGATIVE;
        assert!(!cpu.get_n());
        cpu.change_flag_negative(-1i8 as u8);
        assert!(cpu.get_n());
    }

    #[test]
    fn flag_not_negative() {
        let mut cpu = Cpu::new();
        cpu.p = flags::NEGATIVE;
        assert!(cpu.get_n());
        cpu.change_flag_negative(0);
        assert!(!cpu.get_n());
    }
}
