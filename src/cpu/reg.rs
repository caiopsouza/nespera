use std::fmt;

use crate::cpu::cycle;
use crate::cpu::flags;
use crate::cpu::flags::Flags;
use crate::utils::bits;

#[derive(Copy, Clone, PartialEq)]
pub enum InternalOverflow { None, Positive, Negative }

// Registers
#[derive(Clone, PartialEq)]
pub struct Reg {
    // Accumulator
    a: u8,

    // Indexes
    x: u8,
    y: u8,

    // Program counter
    pc: u16,

    // P Flags
    p: Flags,

    // Stack pointer
    s: u8,

    // Temporary variables. Simplify many internal registers of the 6502.
    m: u8,
    n: u8,
    q: u8,

    // Internal operations can overflow. This is saved here to process later.
    internal_overflow: InternalOverflow,

    // Current instruction being executed.
    // Should work the same as the Instruction Register (IR).
    current_instr: u8,

    // Current cycle of the instruction.
    cycle: u16,

    // Data bus. Almost every transfer should pass through here.
    data_bus: u8,

    // Address bus. Every write request should write the address here.
    addr_bus: u16,
}

impl fmt::Debug for Reg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let p = self.get_p();
        let pu8: u8 = p.into();

        write!(formatter,
               "Regs | 0x{:02x}, a: {:02x}, x: {:02x}, y: {:02x}, pc: {:04x}, s: {:02x}, p: {:02x} {}{}{}{}{}{}{}{}",
               self.current_instr,
               self.a, self.x, self.y,
               self.pc, self.s, pu8,
               if p.get_negative() { 'n' } else { '_' },
               if p.get_overflow() { 'v' } else { '_' },
               if p.get_unused() { 'u' } else { '_' },
               if p.get_break_command() { 'b' } else { '_' },
               if p.get_decimal_mode() { 'd' } else { '_' },
               if p.get_interrupt_disable() { 'i' } else { '_' },
               if p.get_zero() { 'z' } else { '_' },
               if p.get_carry() { 'c' } else { '_' })
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}", self)
    }
}

// Type of transfer functions:
//
// - get: Directly get to values.
// - read: Read the data and modify the internal structure of the registers.
//
// - set: Directly set the value.
// - write: Set the data and modify the internal structure of the registers.
//
// - fetch: Reads from PC and advances it
// - prefetch: Reads from PC and keeps its state
impl Reg {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            p: flags::INTERRUPT_DISABLE | flags::UNUSED,
            s: 0,
            m: 0,
            n: 0,
            q: 0,
            internal_overflow: InternalOverflow::None,
            current_instr: 0x00, // BRK. But will actually execute a reset as dictated by the bus
            cycle: cycle::LAST,
            data_bus: 0,
            addr_bus: 0,
        }
    }

    // Some transfers must save their value in the data bus.
    fn db(&mut self, data: u8) -> u8 {
        self.data_bus = data;
        data
    }

    // Set the flags based on whats on the data bus
    fn set_zero_negative_from_db(&mut self) {
        self.p.change_zero_negative(self.data_bus);
    }

    // Getter for the registers
    pub fn get_a(&self) -> u8 { self.a }
    pub fn get_x(&self) -> u8 { self.x }
    pub fn get_y(&self) -> u8 { self.y }
    pub fn get_s(&self) -> u8 { self.s }
    pub fn get_stack_addr(&self) -> u16 { u16::from(self.s) + 0x100 }
    pub fn get_next_stack_addr(&self) -> u16 { u16::from(self.s.wrapping_add(1)) + 0x100 }

    pub fn get_p(&self) -> Flags { self.p }
    pub fn get_p_mut(&mut self) -> &mut Flags { &mut self.p }

    // Readers for the registers
    pub fn read_a(&mut self) -> u8 { self.db(self.a) }
    pub fn read_x(&mut self) -> u8 { self.db(self.x) }
    pub fn read_y(&mut self) -> u8 { self.db(self.y) }
    pub fn read_m(&mut self) -> u8 { self.db(self.m) }

    // Writer for the register
    pub fn write_a(&mut self, data: u8) {
        self.a = self.db(data);
        self.set_zero_negative_from_db();
    }

    pub fn write_x(&mut self, data: u8) {
        self.x = self.db(data);
        self.set_zero_negative_from_db();
    }

    pub fn write_y(&mut self, data: u8) {
        self.y = self.db(data);
        self.set_zero_negative_from_db();
    }

    pub fn write_s(&mut self, data: u8) {
        self.s = self.db(data);
    }

    pub fn write_p(&mut self, data: Flags) {
        self.p = self.db(data.into()).into();
    }

    pub fn write_m(&mut self, data: u8) { self.m = self.db(data) }
    pub fn write_n(&mut self, data: u8) { self.n = self.db(data) }
    pub fn write_q(&mut self, data: u8) { self.q = self.db(data) }

    // Data bus shouldn't change here because this pass though it own bus (ADL and ADH)
    pub fn write_pc(&mut self, data: u16) { self.pc = data; }
    pub fn write_pcl(&mut self, pcl: u8) { self.pc = bits::set_low(self.pc, pcl) }
    pub fn write_pch(&mut self, pch: u8) { self.pc = bits::set_high(self.pc, pch) }

    #[allow(clippy::similar_names)]
    pub fn write_pcl_pch(&mut self, pcl: u8, pch: u8) { self.pc = u16::from(pcl) | (u16::from(pch) << 8); }

    pub fn write_bit_test(&mut self, data: u8) {
        let res = data & self.get_a();
        self.db(res);

        self.p.change(flags::ZERO, res == 0);
        self.p.copy(data.into(), flags::OVERFLOW | flags::NEGATIVE);
    }

    // Instruction
    pub fn get_current_instr(&self) -> u8 { self.current_instr }

    // Cycle
    pub fn get_cycle(&self) -> u16 { self.cycle }
    pub fn is_last_cycle(&self) -> bool { self.cycle == cycle::LAST }
    pub fn set_next_cycle(&mut self) { self.cycle += 1; }
    pub fn set_first_cycle(&mut self) { self.cycle = cycle::FIRST }
    pub fn set_last_cycle(&mut self) { self.cycle = cycle::LAST }
    pub fn set_next_to_last_cycle(&mut self) { self.cycle = cycle::NEX_TO_LAST }

    // PC
    pub fn get_pc(&self) -> u16 { self.pc }
    pub fn get_pcl(&self) -> u8 { self.pc as u8 }
    pub fn get_pch(&self) -> u8 { (self.pc >> 8) as u8 }
    pub fn set_pc(&mut self, pc: u16) { self.pc = pc }
    pub fn set_next_pc(&mut self) { self.pc = self.pc.wrapping_add(1) }

    // Getters for the internal registers
    pub fn get_m(&self) -> u8 { self.m }
    pub fn get_n(&self) -> u8 { self.n }
    pub fn get_q(&self) -> u8 { self.q }
    pub fn get_internal_overflow(&self) -> InternalOverflow { self.internal_overflow }
    pub fn get_absolute(&self) -> u16 { (u16::from(self.n) << 8) | u16::from(self.m) }

    // Setters for the internal registers
    pub fn set_m(&mut self, data: u8) { self.m = data }
    pub fn set_n(&mut self, data: u8) { self.n = data }
    pub fn set_s(&mut self, data: u8) { self.s = data }

    pub fn set_inc_n(&mut self, data: i8) { self.n = self.n.wrapping_add(data as u8); }

    pub fn set_inc_s(&mut self, data: i8) {
        if !cfg!(debug_assertions) {
            self.s = self.s.wrapping_add(data as u8)
        } else if data < 0 {
            let data = -data as u8;
            let (s, overflow) = self.s.overflowing_sub(data as u8);
            self.s = s;
            if overflow { warn!("S overflowed when subtracting {}.", data) }
        } else {
            let data = data as u8;
            let (s, overflow) = self.s.overflowing_add(data as u8);
            self.s = s;
            if overflow { warn!("S overflowed when adding {}.", data) }
        }
    }

    pub fn set_internal_overflow(&mut self, value: InternalOverflow) { self.internal_overflow = value }

    pub fn set_current_instr(&mut self, current_instr: u8) { self.current_instr = current_instr }

    // Writers for internal registers
    pub fn write_inc_m(&mut self, data: u8) {
        let (data, overflow) = self.m.overflowing_add(data);
        self.set_m(data);
        self.internal_overflow = if overflow { InternalOverflow::Positive } else { InternalOverflow::None };
    }

    pub fn write_inc_m_by_x(&mut self) {
        let data = self.read_x();
        self.write_inc_m(data);
    }

    pub fn write_inc_m_by_y(&mut self) {
        let data = self.read_y();
        self.write_inc_m(data);
    }

    // PCL and PCH are represented as a single 16 bit register to ease working with it by other parts.
    // Some bit manipulation is necessary here to write into PCL.
    pub fn write_inc_pcl(&mut self, data: i8) {
        // Transform into PCL then into an i16 for signed manipulation.
        let pcl = i16::from(self.pc as u8);
        let data = i16::from(data);

        let new_pcl = pcl + data;

        // Overflows if any bit is set in the high part of the new PCL.
        self.internal_overflow =
            if ((new_pcl as u16) & 0xff00) == 0 {
                InternalOverflow::None
            } else if data > 0 {
                InternalOverflow::Positive
            } else {
                InternalOverflow::Negative
            };

        // Clear old PCL and set the new one.
        self.pc = (self.pc & 0xff00) | ((new_pcl as u16) & 0x00ff);
    }

    // Setter for address bus
    pub fn addr_bus(&mut self, addr: u16, data: u8) -> u8 {
        self.addr_bus = addr;
        self.data_bus = data;
        self.data_bus
    }

    // Fixes PC based on the internal overflow flag
    pub fn set_fix_carry_pc(&mut self) {
        match self.internal_overflow {
            InternalOverflow::None => {}
            InternalOverflow::Positive => self.pc = self.pc.wrapping_add(0b1_0000_0000),
            InternalOverflow::Negative => self.pc = self.pc.wrapping_sub(0b1_0000_0000),
        }
    }

    // Fixes the N register based on the internal overflow flag
    pub fn set_fix_carry_n(&mut self) {
        match self.internal_overflow {
            InternalOverflow::None => {}
            InternalOverflow::Positive => self.n = self.n.wrapping_add(1),
            InternalOverflow::Negative => self.n = self.n.wrapping_sub(1),
        }
    }

    // Buses' getters
    pub fn get_data_bus(&self) -> u8 { self.data_bus }
    pub fn get_addr_bus(&self) -> u16 { self.addr_bus }
}

// Unsafe setters. These should be used only for debug and testing.
#[cfg(test)]
impl Reg {
    pub fn s_a(&mut self, data: u8) { self.a = data }
    pub fn s_x(&mut self, data: u8) { self.x = data }
    pub fn s_y(&mut self, data: u8) { self.y = data }
    pub fn s_p(&mut self, data: u8) { self.p = Flags::from(data) }
    pub fn s_pc(&mut self, data: u16) { self.pc = data }
    pub fn s_s(&mut self, data: u8) { self.s = data }
    pub fn s_t(&mut self, data: u16) { self.cycle = data }
    pub fn s_c(&mut self, data: bool) { if data { self.p |= flags::CARRY } else { self.p &= !flags::CARRY } }
    pub fn s_z(&mut self, data: bool) { if data { self.p |= flags::ZERO; } else { self.p &= !flags::ZERO } }
    pub fn s_i(&mut self, data: bool) { if data { self.p |= flags::INTERRUPT_DISABLE; } else { self.p &= !flags::INTERRUPT_DISABLE } }
    pub fn s_d(&mut self, data: bool) { if data { self.p |= flags::DECIMAL_MODE; } else { self.p &= !flags::DECIMAL_MODE } }
    pub fn s_b(&mut self, data: bool) { if data { self.p |= flags::BREAK_COMMAND; } else { self.p &= !flags::BREAK_COMMAND } }
    pub fn s_u(&mut self, data: bool) { if data { self.p |= flags::UNUSED; } else { self.p &= !flags::UNUSED } }
    pub fn s_v(&mut self, data: bool) { if data { self.p |= flags::OVERFLOW; } else { self.p &= !flags::OVERFLOW } }
    pub fn s_n(&mut self, data: bool) { if data { self.p |= flags::NEGATIVE; } else { self.p &= !flags::NEGATIVE } }
    pub fn s_instr(&mut self, instr: u8) { self.current_instr = instr }
    pub fn s_db(&mut self, data: u8) { self.data_bus = data }
    pub fn s_ab(&mut self, data: u16) { self.addr_bus = data }
    pub fn s_oper(&mut self, data: u8) { self.m = data }
    pub fn s_oper_other(&mut self, data: u8) { self.n = data }
    pub fn s_oper_v(&mut self, data: InternalOverflow) { self.internal_overflow = data }
}

impl Default for Reg {
    fn default() -> Self { Self::new() }
}