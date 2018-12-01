use std::fmt;

use crate::cpu::bus::Bus;
use crate::cpu::cycle;
use crate::cpu::flags;
use crate::cpu::flags::Flags;

// Registers
#[derive(Clone, PartialOrd, PartialEq)]
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
    internal_overflow: bool,

    // Current instruction being executed.
    // Should work the same as the Instruction Register (IR).
    current_instr: u8,

    // Current cycle of the instruction.
    cycle: u8,

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
//
// - peek: Reads from the address bus.
// - write: Write into the address bus.
impl Reg {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0xfffc,
            p: flags::INTERRUPT_DISABLE | flags::UNUSED,
            s: 0xfd,
            m: 0,
            n: 0,
            q: 0,
            internal_overflow: false,
            current_instr: 0,
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
    pub fn get_stack_addr(&self) -> u16 { self.s as u16 + 0x100 }

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
    pub fn write_pcl(&mut self, pcl: u8) { self.pc = (self.pc & 0xff00) | (pcl as u16); }
    pub fn write_pch(&mut self, pch: u8) { self.pc = (self.pc & 0x00ff) | ((pch as u16) << 8); }
    pub fn write_pcl_pch(&mut self, pcl: u8, pch: u8) { self.pc = (pcl as u16) | ((pch as u16) << 8); }

    pub fn write_bit_test(&mut self, data: u8) {
        let res = data & self.get_a();
        self.db(res);

        self.p.change(flags::ZERO, res == 0);
        self.p.copy(data.into(), flags::OVERFLOW | flags::NEGATIVE);
    }

    // Instruction
    pub fn get_current_instr(&self) -> u8 { self.current_instr }

    // Cycle
    pub fn get_cycle(&self) -> u8 { self.cycle }
    pub fn set_next_cycle(&mut self) { self.cycle += 1; }
    pub fn set_first_cycle(&mut self) { self.cycle = cycle::FIRST }
    pub fn set_next_to_last_cycle(&mut self) { self.cycle = cycle::NEX_TO_LAST }

    // PC
    pub fn get_pc(&self) -> u16 { self.pc }
    pub fn get_pcl(&self) -> u8 { self.pc as u8 }
    pub fn get_pch(&self) -> u8 { (self.pc >> 8) as u8 }
    pub fn peek_pc(&mut self, bus: &mut Bus) -> u8 { self.peek_addr(bus, self.pc) }
    pub fn set_pc(&mut self, pc: u16) { self.pc = pc }
    pub fn set_next_pc(&mut self) { self.pc = self.pc.wrapping_add(1) }
    pub fn fetch_opcode(&mut self, bus: &mut Bus) { self.current_instr = self.fetch_pc(bus) }
    pub fn prefetch_pc(&mut self, bus: &mut Bus) -> u8 { self.peek_addr(bus, self.pc) }
    pub fn fetch_pc(&mut self, bus: &mut Bus) -> u8 {
        let res = self.prefetch_pc(bus);
        self.set_next_pc();
        res
    }

    // Getters for the internal registers
    pub fn get_m(&self) -> u8 { self.m }
    pub fn get_n(&self) -> u8 { self.n }
    pub fn get_q(&self) -> u8 { self.q }
    pub fn get_internal_overflow(&self) -> bool { self.internal_overflow }
    pub fn get_absolute(&self) -> u16 { ((self.n as u16) << 8) | (self.m as u16) }

    // Setters for the internal registers
    fn set_m(&mut self, data: u8, overflow: bool) {
        self.m = data;
        self.internal_overflow = overflow;
    }

    pub fn set_inc_n(&mut self, data: i8) { self.n = self.n.wrapping_add(data as u8) }
    pub fn set_inc_s(&mut self, data: i8) { self.s = self.s.wrapping_add(data as u8) }

    pub fn write_inc_m(&mut self, data: u8) {
        let (data, overflow) = self.m.overflowing_add(data);
        self.set_m(data, overflow);
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
        let pcl = self.pc as u8 as i16;
        let data = data as i16;

        let new_pcl = pcl + data;

        // Overflows if any bit is set in the high part of the new PCL.
        self.internal_overflow = (new_pcl as u16 & 0xff00) != 0;

        // Clear old PCL and set the new one.
        self.pc = (self.pc & 0xff00) | ((new_pcl as u16) & 0x00ff);
    }

    pub fn prefetch_into_m(&mut self, bus: &mut Bus) {
        let data = self.peek_pc(bus);
        self.set_m(data, false)
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

    // Fixes PC based on the internal overflow flag
    pub fn set_fix_carry_pc(&mut self) {
        self.pc = self.pc.wrapping_add((self.internal_overflow as u16) << 8);
    }

    // Fixes the N register based on the internal overflow flag
    pub fn set_fix_carry_n(&mut self) {
        self.n = self.n.wrapping_add(self.internal_overflow as u8);
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

    // Write an address at the specified external bus
    pub fn poke_addr(&mut self, bus: &mut Bus, addr: u16, data: u8) {
        self.addr_bus = addr;
        self.data_bus = data;
        bus.write(addr, data);
    }

    // Read from register as an address to the external bus
    pub fn peek_m(&mut self, bus: &mut Bus) -> u8 {
        self.peek_addr(bus, self.m as u16)
    }

    pub fn peek_at_m(&mut self, bus: &mut Bus, addr: u16) -> u8 {
        let data = self.peek_addr(bus, addr);
        self.write_m(data);
        self.m
    }

    pub fn peek_m_offset(&mut self, bus: &mut Bus, offset: i8) -> u8 {
        self.peek_addr(bus, (self.m.wrapping_add(offset as u8)) as u16)
    }

    pub fn peek_m_to_self(&mut self, bus: &mut Bus) {
        let data = self.peek_addr(bus, self.m as u16);
        self.write_m(data)
    }

    pub fn peek_n_to_self(&mut self, bus: &mut Bus) {
        let data = self.peek_addr(bus, self.n as u16);
        self.write_n(data)
    }

    pub fn peek_m_to_n(&mut self, bus: &mut Bus) {
        let data = self.peek_addr(bus, self.m as u16);
        self.write_n(data)
    }

    pub fn peek_absolute(&mut self, bus: &mut Bus) -> u8 {
        self.peek_addr(bus, self.get_absolute())
    }

    pub fn peek_absolute_to_q(&mut self, bus: &mut Bus) {
        let data = self.peek_absolute(bus);
        self.write_q(data)
    }

    // Write from register as an address to the external bus
    pub fn poke_n_to_m(&mut self, bus: &mut Bus) {
        self.poke_addr(bus, self.m as u16, self.n)
    }

    pub fn poke_q_to_absolute(&mut self, bus: &mut Bus) {
        self.poke_addr(bus, self.get_absolute(), self.get_q())
    }

    // Pushes a value onto the stack
    pub fn poke_at_stack(&mut self, bus: &mut Bus, data: u8) {
        let addr = self.get_stack_addr();
        self.poke_addr(bus, addr, data);
    }

    // Pull a value from the stack
    pub fn peek_at_stack(&mut self, bus: &mut Bus) -> u8 {
        let addr = self.get_stack_addr();
        self.peek_addr(bus, addr)
    }
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
    pub fn s_t(&mut self, data: u8) { self.cycle = data }
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
    pub fn s_oper_v(&mut self, data: bool) { self.internal_overflow = data }
}
