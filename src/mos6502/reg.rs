use crate::mos6502::bus::Bus;
use crate::mos6502::cycle;
use crate::mos6502::cycle::Cycle;
use crate::mos6502::flags;
use crate::mos6502::flags::Flags;

// Registers
#[derive(Debug, Clone, PartialOrd, PartialEq)]
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

    // Internal operations can overflow. This is saved here to process later.
    internal_overflow: bool,

    // Current instruction being executed.
    // Should work the similarly to Instruction Register (IR).
    current_instr: u8,

    // Current cycle of the instruction.
    cycle: Cycle,

    // Data bus. Every transfer should pass though here.
    data_bus: u8,

    // Address bus. Every write request should write the address here.
    addr_bus: u16,
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
//
// - peek: Reads from the address bus.
// - write: Write into the address bus.
impl Reg {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xc000,
            p: Flags::from(flags::INTERRUPT_DISABLE | flags::UNUSED),
            s: 0,
            m: 0,
            n: 0,
            internal_overflow: false,
            current_instr: 0,
            cycle: cycle::LAST,
            data_bus: 0,
            addr_bus: 0,
        }
    }

    // Some transfers must save their value in the data bus.
    fn db(&mut self, value: u8) -> u8 {
        self.data_bus = value;
        value
    }

    // Set the flags based on whats on the data bus
    fn set_zero_negative_from_db(&mut self) {
        self.p = self.p.change(flags::ZERO.into(), self.data_bus == 0);
        self.p = self.p.change(flags::NEGATIVE.into(), self.data_bus & flags::LEAST_BIT != 0);
    }

    // Readers for the registers
    pub fn read_a(&mut self) -> u8 { self.db(self.a) }
    pub fn read_x(&mut self) -> u8 { self.db(self.x) }
    pub fn read_y(&mut self) -> u8 { self.db(self.y) }
    pub fn read_m(&mut self) -> u8 { self.db(self.m) }

    // Writer for the register
    pub fn write_a(&mut self, value: u8) {
        self.a = self.db(value);
        self.set_zero_negative_from_db();
    }

    pub fn write_m(&mut self, value: u8) { self.m = self.db(value) }
    pub fn write_n(&mut self, value: u8) { self.n = self.db(value) }

    // Instruction
    pub fn get_current_instr(&self) -> u8 { self.current_instr }

    // Cycle
    pub fn get_cycle(&self) -> Cycle { self.cycle }
    pub fn set_next_cycle(&mut self) { self.cycle.0 += 1; }
    pub fn set_last_cycle(&mut self) { self.cycle = cycle::LAST }

    // PC
    pub fn get_pc(&self) -> u16 { self.pc }
    pub fn set_next_pc(&mut self) { self.pc = self.pc.wrapping_add(1) }
    pub fn fetch_opcode(&mut self, bus: &mut Bus) {
        self.current_instr = self.peek_pc(bus);
        self.set_next_pc();
    }

    // Getters for the internal registers
    pub fn get_m(&self) -> u8 { self.m }
    pub fn get_n(&self) -> u8 { self.n }
    pub fn get_internal_overflow(&self) -> bool { self.internal_overflow }

    // Setters for the internal registers
    fn set_m(&mut self, value: u8, overflow: bool) {
        self.m = value;
        self.internal_overflow = overflow;
    }

    pub fn write_inc_m(&mut self, value: u8) {
        let (value, overflow) = self.m.overflowing_add(value);
        self.set_m(value, overflow);
    }

    pub fn write_inc_m_by_x(&mut self) {
        let value = self.read_x();
        self.write_inc_m(value);
    }

    pub fn write_inc_m_by_y(&mut self) {
        let value = self.read_y();
        self.write_inc_m(value);
    }

    pub fn prefetch_into_m(&mut self, bus: &mut Bus) {
        let value = self.peek_pc(bus);
        self.set_m(value, false)
    }

    pub fn fetch_into_m(&mut self, bus: &mut Bus) -> u8 {
        self.prefetch_into_m(bus);
        self.set_next_pc();
        self.m
    }

    pub fn fetch_into_n(&mut self, bus: &mut Bus) -> u8 {
        self.n = self.peek_pc(bus);
        self.set_next_pc();
        self.n
    }

    // Fixes the N based on the internal overflow flag
    pub fn set_fix_carry_n(&mut self) {
        self.n = self.n.wrapping_add(self.internal_overflow as u8);
        self.internal_overflow = false;
    }

    // Buses' getters
    pub fn get_data_bus(&self) -> u8 { self.data_bus }
    pub fn get_addr_bus(&self) -> u16 { self.addr_bus }

    // Reads an address at the specified external bus
    pub fn peek_addr(&mut self, bus: &mut Bus, addr: u16) -> u8 {
        self.addr_bus = addr;
        self.data_bus = bus.read(addr);
        self.data_bus
    }

    // Read the PC at the external bus
    pub fn peek_pc(&mut self, bus: &mut Bus) -> u8 { self.peek_addr(bus, self.pc) }

    // Read from M as an address to the external bus
    pub fn peek_m(&mut self, bus: &mut Bus) -> u8 {
        self.peek_addr(bus, self.m as u16)
    }

    pub fn peek_m_offset(&mut self, bus: &mut Bus, offset: i8) -> u8 {
        self.peek_addr(bus, (self.m.wrapping_add(offset as u8)) as u16)
    }

    pub fn peek_m_at_self(&mut self, bus: &mut Bus) {
        let value = self.peek_addr(bus, self.m as u16);
        self.write_m(value)
    }

    pub fn peek_n_at_self(&mut self, bus: &mut Bus) {
        let value = self.peek_addr(bus, self.n as u16);
        self.write_n(value)
    }

    pub fn peek_absolute(&mut self, bus: &mut Bus) -> u8 {
        self.peek_addr(bus, (self.m as u16) | (self.n as u16) << 8)
    }
}

// Unsafe setters. These should be used only for debug and testing.
#[cfg(test)]
impl Reg {
    pub fn s_a(&mut self, value: u8) { self.a = value }
    pub fn s_x(&mut self, value: u8) { self.x = value }
    pub fn s_y(&mut self, value: u8) { self.y = value }
    pub fn s_p(&mut self, value: u8) { self.p = Flags::from(value) }
    pub fn s_pc(&mut self, value: u16) { self.pc = value }
    pub fn s_s(&mut self, value: u8) { self.s = value }
    pub fn s_t(&mut self, value: u8) { self.cycle = Cycle(value) }
    pub fn s_c(&mut self, value: bool) { if value { self.p.0 |= flags::CARRY } else { self.p.0 &= !flags::CARRY } }
    pub fn s_z(&mut self, value: bool) { if value { self.p.0 |= flags::ZERO; } else { self.p.0 &= !flags::ZERO } }
    pub fn s_i(&mut self, value: bool) { if value { self.p.0 |= flags::INTERRUPT_DISABLE; } else { self.p.0 &= !flags::INTERRUPT_DISABLE } }
    pub fn s_d(&mut self, value: bool) { if value { self.p.0 |= flags::DECIMAL_MODE; } else { self.p.0 &= !flags::DECIMAL_MODE } }
    pub fn s_b(&mut self, value: bool) { if value { self.p.0 |= flags::BREAK_COMMAND; } else { self.p.0 &= !flags::BREAK_COMMAND } }
    pub fn s_u(&mut self, value: bool) { if value { self.p.0 |= flags::UNUSED; } else { self.p.0 &= !flags::UNUSED } }
    pub fn s_v(&mut self, value: bool) { if value { self.p.0 |= flags::OVERFLOW; } else { self.p.0 &= !flags::OVERFLOW } }
    pub fn s_n(&mut self, value: bool) { if value { self.p.0 |= flags::NEGATIVE; } else { self.p.0 &= !flags::NEGATIVE } }
    pub fn s_instr(&mut self, instr: u8) { self.current_instr = instr }
    pub fn s_db(&mut self, value: u8) { self.data_bus = value }
    pub fn s_ab(&mut self, value: u16) { self.addr_bus = value }
    pub fn s_oper(&mut self, value: u8) { self.m = value }
    pub fn s_oper_other(&mut self, value: u8) { self.n = value }
    pub fn s_oper_v(&mut self, value: bool) { self.internal_overflow = value }
}
