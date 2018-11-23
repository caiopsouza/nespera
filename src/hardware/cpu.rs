use std::fmt;

use crate::hardware::flags;

#[derive(PartialOrd, PartialEq, Copy, Clone)]
pub struct Cpu {
    // Accumulator
    a: u8,

    // Index X
    x: u8,

    // Index Y
    y: u8,

    // Flags
    p: u8,

    // Program counter
    pc: u16,

    // Stack pointer
    s: u8,
}

impl Cpu {
    // Create a new CPU. It has the default "power up" state.
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xc000,
            s: 0,
            p: flags::INTERRUPT_DISABLE | flags::UNUSED,
        }
    }

    // Dismember 16 bits into its bytes components
    pub fn lsb(value: u16) -> u8 { ((value) >> 8) as u8 }
    pub fn msb(value: u16) -> u8 { value as u8 }

    // Copy a byte into a 16 bits value
    pub fn copy_lsb(value: u16, byte: u8) -> u16 { (value & 0x00ff) | ((byte as u16) << 8) }
    pub fn copy_msb(value: u16, byte: u8) -> u16 { (value & 0xff00) | (byte as u16) }

    // Getters for the registers
    pub fn get_a(&self) -> u8 { self.a }
    pub fn get_x(&self) -> u8 { self.x }
    pub fn get_y(&self) -> u8 { self.y }

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
    pub fn set_p(&mut self, flag: u8) { self.p = flag; }

    // Getters for PC
    pub fn get_pc(&self) -> u16 { self.pc }
    pub fn get_pcl(&self) -> u8 { Self::lsb(self.pc) }
    pub fn get_pch(&self) -> u8 { Self::msb(self.pc) }

    // Setters for PC
    pub fn set_pc(&mut self, value: u16) { self.pc = value; }
    pub fn set_pcl(&mut self, value: u8) { self.pc = Self::copy_lsb(self.pc, value); }
    pub fn set_pch(&mut self, value: u8) { self.pc = Self::copy_msb(self.pc, value); }

    // Increment PC
    pub fn inc_pc(&mut self) { self.set_pc(self.pc.wrapping_add(1)); }

    // Setters for the register
    pub fn set_a(&mut self, value: u8) {
        self.p = flags::change_zero_negative(self.p, value);
        self.a = value;
    }

    pub fn set_x(&mut self, value: u8) {
        self.p = flags::change_zero_negative(self.p, value);
        self.x = value;
    }

    pub fn set_y(&mut self, value: u8) {
        self.p = flags::change_zero_negative(self.p, value);
        self.y = value;
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

// Unsafe setters. These should be used only for debug and testing.
#[cfg(test)]
pub trait Testing {
    fn s_a(&mut self, value: u8);
    fn s_x(&mut self, value: u8);
    fn s_y(&mut self, value: u8);
    fn s_p(&mut self, value: u8);
    fn s_pc(&mut self, value: u16);
    fn s_s(&mut self, value: u8);
    fn s_c(&mut self, value: bool);
    fn s_z(&mut self, value: bool);
    fn s_i(&mut self, value: bool);
    fn s_d(&mut self, value: bool);
    fn s_b(&mut self, value: bool);
    fn s_u(&mut self, value: bool);
    fn s_v(&mut self, value: bool);
    fn s_n(&mut self, value: bool);
}

#[cfg(test)]
impl Testing for Cpu {
    fn s_a(&mut self, value: u8) { self.a = value }
    fn s_x(&mut self, value: u8) { self.x = value }
    fn s_y(&mut self, value: u8) { self.y = value }
    fn s_p(&mut self, value: u8) { self.p = value }
    fn s_pc(&mut self, value: u16) { self.pc = value }
    fn s_s(&mut self, value: u8) { self.s = value }
    fn s_c(&mut self, value: bool) { if value { self.p |= flags::CARRY } else { self.p &= !flags::CARRY } }
    fn s_z(&mut self, value: bool) { if value { self.p |= flags::ZERO; } else { self.p &= !flags::ZERO } }
    fn s_i(&mut self, value: bool) { if value { self.p |= flags::INTERRUPT_DISABLE; } else { self.p &= !flags::INTERRUPT_DISABLE } }
    fn s_d(&mut self, value: bool) { if value { self.p |= flags::DECIMAL_MODE; } else { self.p &= !flags::DECIMAL_MODE } }
    fn s_b(&mut self, value: bool) { if value { self.p |= flags::BREAK_COMMAND; } else { self.p &= !flags::BREAK_COMMAND } }
    fn s_u(&mut self, value: bool) { if value { self.p |= flags::UNUSED; } else { self.p &= !flags::UNUSED } }
    fn s_v(&mut self, value: bool) { if value { self.p |= flags::OVERFLOW; } else { self.p &= !flags::OVERFLOW } }
    fn s_n(&mut self, value: bool) { if value { self.p |= flags::NEGATIVE; } else { self.p &= !flags::NEGATIVE } }
}
