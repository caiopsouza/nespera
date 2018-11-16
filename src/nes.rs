use cpu::Cpu;
use opc;
use flags::Flags;
use std::num::Wrapping;
use std::fmt;
use pretty_hex::*;
use std::ops::*;

// RAM
const RAM_CAPACITY: usize = 0x0800;

#[derive(Copy, Clone)]
pub struct Ram(pub [u8; RAM_CAPACITY]);

// NES
#[derive(Copy, Clone)]
pub struct Nes {
    pub cpu: Cpu,
    pub ram: Ram,
}

impl fmt::Debug for Nes {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter,
               "Nespera | a: {:x}, x: {:x}, y: {:x}, pc: {:x}, sp: {:x}, flags: {}{}{}{}{}{}{}{}\n",
               self.cpu.a, self.cpu.x, self.cpu.y, self.cpu.pc, self.cpu.sp,
               if self.get_c() { 'c' } else { '_' },
               if self.get_z() { 'z' } else { '_' },
               if self.get_i() { 'i' } else { '_' },
               if self.get_d() { 'd' } else { '_' },
               if self.get_b() { 'b' } else { '_' },
               '_' /* Unused*/,
               if self.get_o() { 'o' } else { '_' },
               if self.get_n() { 'n' } else { '_' })?;
        write!(formatter, "{:?}", (&self.ram.0[..]).hex_dump())
    }
}

impl Nes {
    // Constructor
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            ram: Ram([0; RAM_CAPACITY]),
        }
    }

    // Getters for registers
    pub fn get_a(&self) -> u8 { self.cpu.a }
    pub fn get_x(&self) -> u8 { self.cpu.x }
    pub fn get_y(&self) -> u8 { self.cpu.y }
    pub fn get_p(&self) -> u8 { self.cpu.p.bits() }
    pub fn get_pc(&self) -> u16 { self.cpu.pc }
    pub fn get_sp(&self) -> u8 { self.cpu.sp }

    // Getters for flags
    pub fn get_c(&self) -> bool { self.cpu.p.intersects(Flags::Carry) }
    pub fn get_z(&self) -> bool { self.cpu.p.intersects(Flags::Zero) }
    pub fn get_i(&self) -> bool { self.cpu.p.intersects(Flags::InterruptDisable) }
    pub fn get_d(&self) -> bool { self.cpu.p.intersects(Flags::DecimalMode) }
    pub fn get_b(&self) -> bool { self.cpu.p.intersects(Flags::BreakCommand) }
    pub fn get_u(&self) -> bool { self.cpu.p.intersects(Flags::Unused) }
    pub fn get_o(&self) -> bool { self.cpu.p.intersects(Flags::Overflow) }
    pub fn get_n(&self) -> bool { self.cpu.p.intersects(Flags::Negative) }

    // Read the value in RAM pointed by the address
    pub fn peek_at(&self, addr: u16) -> u8 {
        self.ram.0[addr as usize]
    }

    // Read the value in RAM pointed by PC without changing it
    pub fn peek(&self) -> u8 {
        self.peek_at(self.get_pc())
    }

    // Write the value to RAM pointed by the address
    pub fn put_at(&mut self, addr: u16, value: u8) {
        self.ram.0[addr as usize] = value;
    }

    // Write the value to RAM pointed by PC without changing it
    pub fn put(&mut self, value: u8) {
        let addr = self.get_pc();
        self.put_at(addr, value);
    }

    // Read the value in RAM pointed by PC and advances it
    pub fn fetch(&mut self) -> u8 {
        let data = self.peek();
        self.cpu.inc_pc();
        data
    }

    // Write the value to RAM pointed by PC and advances it
    pub fn write(&mut self, value: u8) {
        self.put(value);
        self.cpu.inc_pc();
    }

    // Value getters
    fn immediate(&mut self) -> u8 { self.fetch() }

    // Address getters
    fn zero_page(&mut self) -> u16 { self.fetch().into() }

    fn zero_page_x(&mut self) -> u16 { (Wrapping(self.fetch()) + Wrapping(self.cpu.x)).0.into() }

    fn zero_page_y(&mut self) -> u16 { (Wrapping(self.fetch()) + Wrapping(self.cpu.y)).0.into() }

    fn absolute(&mut self) -> u16 {
        let lsb = self.fetch() as u16;
        let msb = (self.fetch() as u16) << 8;
        msb + lsb
    }

    fn absolute_x(&mut self) -> u16 {
        let lsb = self.fetch() as u16;
        let msb = (self.fetch() as u16) << 8;
        (Wrapping(msb + lsb) + Wrapping(self.cpu.x as u16)).0
    }

    fn absolute_y(&mut self) -> u16 {
        let lsb = self.fetch() as u16;
        let msb = (self.fetch() as u16) << 8;
        (Wrapping(msb + lsb) + Wrapping(self.cpu.y as u16)).0
    }

    fn indirect_x(&mut self) -> u16 {
        let addr = Wrapping(self.fetch()) + Wrapping(self.cpu.x);
        let lsb = self.peek_at(addr.0.into()) as u16;
        let msb = (self.peek_at((addr + Wrapping(1)).0.into()) as u16) << 8;
        msb + lsb
    }

    fn indirect_y(&mut self) -> u16 {
        let addr = Wrapping(self.fetch());
        let lsb = self.peek_at(addr.0.into()) as u16;
        let msb = (self.peek_at((addr + Wrapping(1)).0.into()) as u16) << 8;
        (Wrapping(msb + lsb) + Wrapping(self.cpu.y as u16)).0
    }

    // Executes one step of the CPU
    pub fn step(&mut self) {
        // Set a value through another
        macro_rules! set_reg_val {
            ( $setter:ident, $getter:ident ) => {{
                let value = self.$getter();
                self.cpu.$setter(value);
            }};
        }

        // Set a value through an address
        macro_rules! set_reg_addr {
            ( $setter:ident, $getter:ident ) => {{
                let addr = self.$getter();
                let value = self.peek_at(addr);
                self.cpu.$setter(value);
            }};
        }

        // Set a value at an address
        macro_rules! set_mem_addr {
            ( $setter:ident, $getter:ident ) => {{
                let addr = self.$setter();
                let value = self.$getter();
                self.put_at(addr, value);
            }};
        }

        // Push a value
        macro_rules! push_reg_val {
            ( $getter:ident ) => {{
                let addr = self.cpu.sp;
                let value = self.$getter();
                self.put_at(addr.into(), value);
                self.cpu.sp -= 1;
            }};
        }

        // Pull a value referenced through it's address
        macro_rules! pull_reg_val {
            ( $setter:ident ) => {{
                let value = self.peek_at(self.cpu.sp.into());
                self.cpu.$setter(value);
                self.cpu.sp += 1;
            }};
        }

        // Operator on a value
        macro_rules! set_op_val {
            ( $op:ident, $getter:ident ) => {{
                let value = self.cpu.a.$op(self.$getter());
                self.cpu.set_a(value);
            }};
        }

        // Operator on a value through it's address
        macro_rules! set_op_addr {
            ( $op:ident, $getter:ident ) => {{
                let addr = self.$getter();
                let value = self.cpu.a.$op(self.peek_at(addr));
                self.cpu.set_a(value);
            }};
        }

        // Bit test
        macro_rules! bit_test {
            ( $getter:ident ) => {{
                let addr = self.$getter();
                let value = self.cpu.a & self.peek_at(addr);
                self.cpu.p.zno_bit_test(value);
            }};
        }

        let opcode = self.fetch();
        match opcode {
            // Load into A
            opc::Lda::Immediate => set_reg_val!(set_a, immediate),
            opc::Lda::ZeroPage => set_reg_addr!(set_a, zero_page),
            opc::Lda::ZeroPageX => set_reg_addr!(set_a, zero_page_x),
            opc::Lda::Absolute => set_reg_addr!(set_a, absolute),
            opc::Lda::AbsoluteX => set_reg_addr!(set_a, absolute_x),
            opc::Lda::AbsoluteY => set_reg_addr!(set_a, absolute_y),
            opc::Lda::IndirectX => set_reg_addr!(set_a, indirect_x),
            opc::Lda::IndirectY => set_reg_addr!(set_a, indirect_y),

            // Load into X
            opc::Ldx::Immediate => set_reg_val!(set_x, immediate),
            opc::Ldx::ZeroPage => set_reg_addr!(set_x, zero_page),
            opc::Ldx::ZeroPageY => set_reg_addr!(set_x, zero_page_y),
            opc::Ldx::Absolute => set_reg_addr!(set_x, absolute),
            opc::Ldx::AbsoluteY => set_reg_addr!(set_x, absolute_y),

            // Load into Y
            opc::Ldy::Immediate => set_reg_val!(set_y, immediate),
            opc::Ldy::ZeroPage => set_reg_addr!(set_y, zero_page),
            opc::Ldy::ZeroPageX => set_reg_addr!(set_y, zero_page_x),
            opc::Ldy::Absolute => set_reg_addr!(set_y, absolute),
            opc::Ldy::AbsoluteX => set_reg_addr!(set_y, absolute_x),

            // Store from A
            opc::Sta::ZeroPage => set_mem_addr!(zero_page, get_a),
            opc::Sta::ZeroPageX => set_mem_addr!(zero_page_x, get_a),
            opc::Sta::Absolute => set_mem_addr!(absolute, get_a),
            opc::Sta::AbsoluteX => set_mem_addr!(absolute_x, get_a),
            opc::Sta::AbsoluteY => set_mem_addr!(absolute_y, get_a),
            opc::Sta::IndirectX => set_mem_addr!(indirect_x, get_a),
            opc::Sta::IndirectY => set_mem_addr!(indirect_y, get_a),

            // Store from X
            opc::Stx::ZeroPage => set_mem_addr!(zero_page, get_x),
            opc::Stx::ZeroPageY => set_mem_addr!(zero_page_y, get_x),
            opc::Stx::Absolute => set_mem_addr!(absolute, get_x),

            // Store from Y
            opc::Sty::ZeroPage => set_mem_addr!(zero_page, get_y),
            opc::Sty::ZeroPageX => set_mem_addr!(zero_page_x, get_y),
            opc::Sty::Absolute => set_mem_addr!(absolute, get_y),

            // Transfer
            opc::Tax => set_reg_val!(set_x, get_a),
            opc::Tay => set_reg_val!(set_y, get_a),
            opc::Txa => set_reg_val!(set_a, get_x),
            opc::Tya => set_reg_val!(set_a, get_y),
            opc::Tsx => set_reg_val!(set_x, get_sp),
            opc::Txs => set_reg_val!(set_sp, get_x),

            // Stack
            opc::Pha => push_reg_val!(get_a),
            opc::Php => push_reg_val!(get_p),
            opc::Pla => pull_reg_val!(set_a),
            opc::Plp => pull_reg_val!(set_p),

            // And
            opc::And::Immediate => set_op_val!(bitand, immediate),
            opc::And::ZeroPage => set_op_addr!(bitand, zero_page),
            opc::And::ZeroPageX => set_op_addr!(bitand, zero_page_x),
            opc::And::Absolute => set_op_addr!(bitand, absolute),
            opc::And::AbsoluteX => set_op_addr!(bitand, absolute_x),
            opc::And::AbsoluteY => set_op_addr!(bitand, absolute_y),
            opc::And::IndirectX => set_op_addr!(bitand, indirect_x),
            opc::And::IndirectY => set_op_addr!(bitand, indirect_y),

            // Or
            opc::Ora::Immediate => set_op_val!(bitor, immediate),
            opc::Ora::ZeroPage => set_op_addr!(bitor, zero_page),
            opc::Ora::ZeroPageX => set_op_addr!(bitor, zero_page_x),
            opc::Ora::Absolute => set_op_addr!(bitor, absolute),
            opc::Ora::AbsoluteX => set_op_addr!(bitor, absolute_x),
            opc::Ora::AbsoluteY => set_op_addr!(bitor, absolute_y),
            opc::Ora::IndirectX => set_op_addr!(bitor, indirect_x),
            opc::Ora::IndirectY => set_op_addr!(bitor, indirect_y),

            // Xor
            opc::Eor::Immediate => set_op_val!(bitxor, immediate),
            opc::Eor::ZeroPage => set_op_addr!(bitxor, zero_page),
            opc::Eor::ZeroPageX => set_op_addr!(bitxor, zero_page_x),
            opc::Eor::Absolute => set_op_addr!(bitxor, absolute),
            opc::Eor::AbsoluteX => set_op_addr!(bitxor, absolute_x),
            opc::Eor::AbsoluteY => set_op_addr!(bitxor, absolute_y),
            opc::Eor::IndirectX => set_op_addr!(bitxor, indirect_x),
            opc::Eor::IndirectY => set_op_addr!(bitxor, indirect_y),

            // Bit test
            opc::Bit::ZeroPage => bit_test!(zero_page),
            opc::Bit::Absolute => bit_test!(absolute),

            // Not implemented
            _ => panic!("Opcode not implemented: {:x?}", opcode)
        }
    }
}
