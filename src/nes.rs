use cpu::Cpu;
use cpu::Flags;
use opc;
use std::num::Wrapping;
use std::fmt;
use pretty_hex::*;
use std::ops::*;
use ram::Ram;

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
               if self.cpu.get_c() { 'c' } else { '_' },
               if self.cpu.get_z() { 'z' } else { '_' },
               if self.cpu.get_i() { 'i' } else { '_' },
               if self.cpu.get_d() { 'd' } else { '_' },
               if self.cpu.get_b() { 'b' } else { '_' },
               '_' /* Unused*/,
               if self.cpu.get_v() { 'v' } else { '_' },
               if self.cpu.get_n() { 'n' } else { '_' })?;
        write!(formatter, "{:?}", (&self.ram.0[..]).hex_dump())
    }
}

impl Nes {
    // Constructor
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            ram: Ram::new(),
        }
    }

    // Read the value in RAM pointed by PC and advances it
    pub fn fetch(&mut self) -> u8 {
        let data = self.ram.peek_at(self.cpu.pc);
        self.cpu.inc_pc();
        data
    }

    // Read the value in RAM pointed by PC as an i16 and advances it
    pub fn fetch_16(&mut self) -> u16 {
        let pc = self.cpu.pc;
        self.cpu.pc = (Wrapping(self.cpu.pc) + Wrapping(2)).0;
        self.ram.peek_at_16(pc)
    }

    // Address getters
    fn immediate(&mut self) -> u16 {
        let pc = self.cpu.pc;
        self.cpu.inc_pc();
        pc
    }

    fn zero_page(&mut self) -> u16 { self.fetch().into() }

    fn zero_page_x(&mut self) -> u16 { (Wrapping(self.fetch()) + Wrapping(self.cpu.x)).0.into() }

    fn zero_page_y(&mut self) -> u16 { (Wrapping(self.fetch()) + Wrapping(self.cpu.y)).0.into() }

    fn absolute(&mut self) -> u16 { self.fetch_16() }

    fn absolute_x(&mut self) -> u16 {
        let addr = self.fetch_16();
        (Wrapping(addr) + Wrapping(self.cpu.x as u16)).0
    }

    fn absolute_y(&mut self) -> u16 {
        let addr = self.fetch_16();
        (Wrapping(addr) + Wrapping(self.cpu.y as u16)).0
    }

    fn indirect(&mut self) -> u16 {
        let mut addr = self.fetch_16();
        addr = self.ram.peek_at_16(addr);
        self.ram.peek_at_16(addr)
    }

    fn indirect_x(&mut self) -> u16 {
        let addr = (Wrapping(self.fetch()) + Wrapping(self.cpu.x)).0 as u16;
        self.ram.peek_at_16(addr)
    }

    fn indirect_y(&mut self) -> u16 {
        let addr = self.fetch();
        let addr = self.ram.peek_at_16(addr.into());
        (Wrapping(addr) + Wrapping(self.cpu.y as u16)).0
    }

    // Executes one step of the CPU
    pub fn step(&mut self) {
        // Set a value through another
        macro_rules! set_reg_val {
            ( $setter:ident, $getter:ident ) => {{
                let value = self.cpu.$getter();
                self.cpu.$setter(value);
            }};
        }

        // Set a value through an address
        macro_rules! set_reg {
            ( $setter:ident, $getter:ident ) => {{
                let addr = self.$getter();
                let value = self.ram.peek_at(addr);
                self.cpu.$setter(value);
            }};
        }

        // Set a value at an address
        macro_rules! set_mem {
            ( $setter:ident, $getter:ident ) => {{
                let addr = self.$setter();
                let value = self.cpu.$getter();
                self.ram.put_at(addr, value);
            }};
        }

        // Push a value
        macro_rules! push_reg {
            ( $getter:ident ) => {{
                let addr = self.cpu.sp;
                let value = self.cpu.$getter();
                self.ram.put_at(addr.into(), value);
                self.cpu.sp -= 1;
            }};
        }

        // Pull a value referenced through it's address
        macro_rules! pull_reg {
            ( $setter:ident ) => {{
                let value = self.ram.peek_at(self.cpu.sp.into());
                self.cpu.$setter(value);
                self.cpu.sp += 1;
            }};
        }

        // Operator on a value through it's address
        macro_rules! set_op {
            ( $op:ident, $getter:ident ) => {{
                let addr = self.$getter();
                let value = self.cpu.a.$op(self.ram.peek_at(addr));
                self.cpu.set_a(value);
            }};
        }

        // Bit test
        macro_rules! bit_test {
            ( $getter:ident ) => {{
                let addr = self.$getter();
                let value = self.cpu.a & self.ram.peek_at(addr);
                self.cpu.p.znv_bit_test(value);
            }};
        }

        // Addition on a value through it's address
        macro_rules! set_adc {
            ( $getter:ident ) => {{
                let addr = self.$getter();
                let value = Wrapping(self.cpu.get_c() as u8) + Wrapping(self.ram.peek_at(addr));
                self.cpu.adc_a(value.0);
            }};
        }

        // Subtraction on a value through it's address
        macro_rules! set_sbc {
            ( $getter:ident ) => {{
                let addr = self.$getter();
                let value = Wrapping(1)
                    - Wrapping(self.cpu.get_c() as u8)
                    + Wrapping(self.ram.peek_at(addr));
                self.cpu.sbc_a(value.0);
            }};
        }

        // Compares
        macro_rules! cmp {
            ( $setter:ident, $getter:ident ) => {{
                let addr = self.$getter();
                let value = self.ram.peek_at(addr);
                self.cpu.$setter(value);
            }}
        }

        // Shift left
        macro_rules! asl {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.ram.peek_at(addr);
                self.cpu.p.znc_left_shift(value);
                self.ram.put_at(addr, value << 1);
            }}
        }

        // Shift Right
        macro_rules! lsr {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.ram.peek_at(addr);
                self.cpu.p.znc_right_shift(value);
                self.ram.put_at(addr, value >> 1);
            }}
        }

        // Rotates left
        macro_rules! rol {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.ram.peek_at(addr);
                self.cpu.p.znc_left_shift(value);
                let carry = self.cpu.p.bits() & Flags::Carry.bits();
                self.ram.put_at(addr, (value << 1) | carry);
            }}
        }

        // Rotates right
        macro_rules! ror {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.ram.peek_at(addr);
                self.cpu.p.znc_left_shift(value);
                let carry = (self.cpu.p.bits() & Flags::Carry.bits()) << 7;
                self.ram.put_at(addr, (value >> 1) | carry);
            }}
        }

        // Increment a register
        macro_rules! inc_mem {
            ( $addr:ident, $step:expr ) => {{
                let addr = self.$addr();
                let value = (Wrapping(self.ram.peek_at(addr)) + Wrapping($step as u8)).0;
                self.cpu.p.zn(value);
                self.ram.put_at(addr, value);
            }}
        }

        // Increment an address
        macro_rules! inc_reg {
            ( $setter:ident, $getter:ident, $step:expr ) => {{
                let value = (Wrapping(self.cpu.$getter()) + Wrapping($step as u8)).0;
                self.cpu.$setter(value);
            }}
        }

        // Branch if the condition is true
        macro_rules! branch {
            ( $condition:expr ) => {{
                let offset = (self.fetch() as i8) as u16;
                if $condition { self.cpu.pc = (Wrapping(self.cpu.pc) + Wrapping(offset)).0; }
            }}
        }

        // Jump
        macro_rules! jump {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                self.cpu.pc = addr;
            }}
        }

        let opcode = self.fetch();
        match opcode {
            // No operation
            opc::Nop => {}

            // Load into A
            opc::Lda::Immediate => set_reg!(set_a, immediate),
            opc::Lda::ZeroPage => set_reg!(set_a, zero_page),
            opc::Lda::ZeroPageX => set_reg!(set_a, zero_page_x),
            opc::Lda::Absolute => set_reg!(set_a, absolute),
            opc::Lda::AbsoluteX => set_reg!(set_a, absolute_x),
            opc::Lda::AbsoluteY => set_reg!(set_a, absolute_y),
            opc::Lda::IndirectX => set_reg!(set_a, indirect_x),
            opc::Lda::IndirectY => set_reg!(set_a, indirect_y),

            // Load into X
            opc::Ldx::Immediate => set_reg!(set_x, immediate),
            opc::Ldx::ZeroPage => set_reg!(set_x, zero_page),
            opc::Ldx::ZeroPageY => set_reg!(set_x, zero_page_y),
            opc::Ldx::Absolute => set_reg!(set_x, absolute),
            opc::Ldx::AbsoluteY => set_reg!(set_x, absolute_y),

            // Load into Y
            opc::Ldy::Immediate => set_reg!(set_y, immediate),
            opc::Ldy::ZeroPage => set_reg!(set_y, zero_page),
            opc::Ldy::ZeroPageX => set_reg!(set_y, zero_page_x),
            opc::Ldy::Absolute => set_reg!(set_y, absolute),
            opc::Ldy::AbsoluteX => set_reg!(set_y, absolute_x),

            // Store from A
            opc::Sta::ZeroPage => set_mem!(zero_page, get_a),
            opc::Sta::ZeroPageX => set_mem!(zero_page_x, get_a),
            opc::Sta::Absolute => set_mem!(absolute, get_a),
            opc::Sta::AbsoluteX => set_mem!(absolute_x, get_a),
            opc::Sta::AbsoluteY => set_mem!(absolute_y, get_a),
            opc::Sta::IndirectX => set_mem!(indirect_x, get_a),
            opc::Sta::IndirectY => set_mem!(indirect_y, get_a),

            // Store from X
            opc::Stx::ZeroPage => set_mem!(zero_page, get_x),
            opc::Stx::ZeroPageY => set_mem!(zero_page_y, get_x),
            opc::Stx::Absolute => set_mem!(absolute, get_x),

            // Store from Y
            opc::Sty::ZeroPage => set_mem!(zero_page, get_y),
            opc::Sty::ZeroPageX => set_mem!(zero_page_x, get_y),
            opc::Sty::Absolute => set_mem!(absolute, get_y),

            // Transfer
            opc::Tax => set_reg_val!(set_x, get_a),
            opc::Tay => set_reg_val!(set_y, get_a),
            opc::Txa => set_reg_val!(set_a, get_x),
            opc::Tya => set_reg_val!(set_a, get_y),
            opc::Tsx => set_reg_val!(set_x, get_sp),
            opc::Txs => set_reg_val!(set_sp, get_x),

            // Stack
            opc::Pha => push_reg!(get_a),
            opc::Php => push_reg!(get_p),
            opc::Pla => pull_reg!(set_a),
            opc::Plp => pull_reg!(set_p),

            // And
            opc::And::Immediate => set_op!(bitand, immediate),
            opc::And::ZeroPage => set_op!(bitand, zero_page),
            opc::And::ZeroPageX => set_op!(bitand, zero_page_x),
            opc::And::Absolute => set_op!(bitand, absolute),
            opc::And::AbsoluteX => set_op!(bitand, absolute_x),
            opc::And::AbsoluteY => set_op!(bitand, absolute_y),
            opc::And::IndirectX => set_op!(bitand, indirect_x),
            opc::And::IndirectY => set_op!(bitand, indirect_y),

            // Or
            opc::Ora::Immediate => set_op!(bitor, immediate),
            opc::Ora::ZeroPage => set_op!(bitor, zero_page),
            opc::Ora::ZeroPageX => set_op!(bitor, zero_page_x),
            opc::Ora::Absolute => set_op!(bitor, absolute),
            opc::Ora::AbsoluteX => set_op!(bitor, absolute_x),
            opc::Ora::AbsoluteY => set_op!(bitor, absolute_y),
            opc::Ora::IndirectX => set_op!(bitor, indirect_x),
            opc::Ora::IndirectY => set_op!(bitor, indirect_y),

            // Xor
            opc::Eor::Immediate => set_op!(bitxor, immediate),
            opc::Eor::ZeroPage => set_op!(bitxor, zero_page),
            opc::Eor::ZeroPageX => set_op!(bitxor, zero_page_x),
            opc::Eor::Absolute => set_op!(bitxor, absolute),
            opc::Eor::AbsoluteX => set_op!(bitxor, absolute_x),
            opc::Eor::AbsoluteY => set_op!(bitxor, absolute_y),
            opc::Eor::IndirectX => set_op!(bitxor, indirect_x),
            opc::Eor::IndirectY => set_op!(bitxor, indirect_y),

            // Bit test
            opc::Bit::ZeroPage => bit_test!(zero_page),
            opc::Bit::Absolute => bit_test!(absolute),

            // Addition
            opc::Adc::Immediate => set_adc!(immediate),
            opc::Adc::ZeroPage => set_adc!(zero_page),
            opc::Adc::ZeroPageX => set_adc!(zero_page_x),
            opc::Adc::Absolute => set_adc!(absolute),
            opc::Adc::AbsoluteX => set_adc!(absolute_x),
            opc::Adc::AbsoluteY => set_adc!(absolute_y),
            opc::Adc::IndirectX => set_adc!(indirect_x),
            opc::Adc::IndirectY => set_adc!(indirect_y),

            // Subtraction
            opc::Sbc::Immediate => set_sbc!(immediate),
            opc::Sbc::ZeroPage => set_sbc!(zero_page),
            opc::Sbc::ZeroPageX => set_sbc!(zero_page_x),
            opc::Sbc::Absolute => set_sbc!(absolute),
            opc::Sbc::AbsoluteX => set_sbc!(absolute_x),
            opc::Sbc::AbsoluteY => set_sbc!(absolute_y),
            opc::Sbc::IndirectX => set_sbc!(indirect_x),
            opc::Sbc::IndirectY => set_sbc!(indirect_y),

            // Increment in memory
            opc::Inc::ZeroPage => inc_mem!(zero_page, 1i8),
            opc::Inc::ZeroPageX => inc_mem!(zero_page_x, 1i8),
            opc::Inc::Absolute => inc_mem!(absolute, 1i8),
            opc::Inc::AbsoluteX => inc_mem!(absolute_x, 1i8),

            // Decrement in memory
            opc::Dec::ZeroPage => inc_mem!(zero_page, -1i8),
            opc::Dec::ZeroPageX => inc_mem!(zero_page_x, -1i8),
            opc::Dec::Absolute => inc_mem!(absolute, -1i8),
            opc::Dec::AbsoluteX => inc_mem!(absolute_x, -1i8),

            // Increment and decrement registers
            opc::Inx => inc_reg!(set_x, get_x, 1i8),
            opc::Dex => inc_reg!(set_x, get_x, -1i8),
            opc::Iny => inc_reg!(set_y, get_y, 1i8),
            opc::Dey => inc_reg!(set_y, get_y, -1i8),

            // Shifts Left
            opc::Asl::Accumulator => self.cpu.shift_a_left(),
            opc::Asl::ZeroPage => asl!(zero_page),
            opc::Asl::ZeroPageX => asl!(zero_page_x),
            opc::Asl::Absolute => asl!(absolute),
            opc::Asl::AbsoluteX => asl!(absolute_x),

            // Shifts Right
            opc::Lsr::Accumulator => self.cpu.shift_a_right(),
            opc::Lsr::ZeroPage => lsr!(zero_page),
            opc::Lsr::ZeroPageX => lsr!(zero_page_x),
            opc::Lsr::Absolute => lsr!(absolute),
            opc::Lsr::AbsoluteX => lsr!(absolute_x),

            // Rotates Left
            opc::Rol::Accumulator => self.cpu.rotate_a_left(),
            opc::Rol::ZeroPage => rol!(zero_page),
            opc::Rol::ZeroPageX => rol!(zero_page_x),
            opc::Rol::Absolute => rol!(absolute),
            opc::Rol::AbsoluteX => rol!(absolute_x),

            // Rotates Right
            opc::Ror::Accumulator => self.cpu.rotate_a_right(),
            opc::Ror::ZeroPage => ror!(zero_page),
            opc::Ror::ZeroPageX => ror!(zero_page_x),
            opc::Ror::Absolute => ror!(absolute),
            opc::Ror::AbsoluteX => ror!(absolute_x),

            // Compare A
            opc::Cmp::Immediate => cmp!(cmp_a, immediate),
            opc::Cmp::ZeroPage => cmp!(cmp_a, zero_page),
            opc::Cmp::ZeroPageX => cmp!(cmp_a, zero_page_x),
            opc::Cmp::Absolute => cmp!(cmp_a, absolute),
            opc::Cmp::AbsoluteX => cmp!(cmp_a, absolute_x),
            opc::Cmp::AbsoluteY => cmp!(cmp_a, absolute_y),
            opc::Cmp::IndirectX => cmp!(cmp_a, indirect_x),
            opc::Cmp::IndirectY => cmp!(cmp_a, indirect_y),

            // Compare X
            opc::Cpx::Immediate => cmp!(cmp_x, immediate),
            opc::Cpx::ZeroPage => cmp!(cmp_x, zero_page),
            opc::Cpx::Absolute => cmp!(cmp_x, absolute),

            // Compare X
            opc::Cpy::Immediate => cmp!(cmp_y, immediate),
            opc::Cpy::ZeroPage => cmp!(cmp_y, zero_page),
            opc::Cpy::Absolute => cmp!(cmp_y, absolute),

            // Status flags
            opc::Clc => self.cpu.clear_flag(Flags::Carry),
            opc::Cld => self.cpu.clear_flag(Flags::DecimalMode),
            opc::Cli => self.cpu.clear_flag(Flags::InterruptDisable),
            opc::Clv => self.cpu.clear_flag(Flags::Overflow),
            opc::Sec => self.cpu.set_flag(Flags::Carry),
            opc::Sed => self.cpu.set_flag(Flags::DecimalMode),
            opc::Sei => self.cpu.set_flag(Flags::InterruptDisable),

            // Branches
            opc::Bcs => branch!(self.cpu.p.contains(Flags::Carry)),
            opc::Bcc => branch!(!self.cpu.p.contains(Flags::Carry)),
            opc::Beq => branch!(self.cpu.p.contains(Flags::Zero)),
            opc::Bne => branch!(!self.cpu.p.contains(Flags::Zero)),
            opc::Bmi => branch!(self.cpu.p.contains(Flags::Negative)),
            opc::Bpl => branch!(!self.cpu.p.contains(Flags::Negative)),
            opc::Bvc => branch!(self.cpu.p.contains(Flags::Overflow)),
            opc::Bvs => branch!(!self.cpu.p.contains(Flags::Overflow)),

            // Jump
            opc::Jmp::Absolute => jump!(absolute),
            opc::Jmp::Indirect => jump!(indirect),

            // Not implemented
            _ => panic!("Opcode not implemented: 0x{:02x?}", opcode)
        }
    }
}
