use hardware::cpu::Cpu;
use hardware::flags::Flags;
use hardware::opc;
use hardware::mem::Memory;

use std::num::Wrapping;
use std::fmt;
use std::ops::*;

// Adds two number wrapping the result
macro_rules! wrap_add {
    ( $first:expr $( ,$rest:expr )* ) => {{
        let mut res = Wrapping($first);
        $( res += Wrapping($rest); )*
        res.0
     }};
}

// NES
#[derive(Copy, Clone)]
pub struct Nes {
    pub cpu: Cpu,
    pub mem: Memory,
}

impl fmt::Debug for Nes {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Nespera | {:?}\n{:?}", &self.cpu, &self.mem)
    }
}

impl Nes {
    // Constructors
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: Memory::new(),
        }
    }

    pub fn new_from_rom(rom: &[u8]) -> Self {
        Self {
            cpu: Cpu::new_from_pc(0xc000),
            mem: Memory::new_from_rom(rom),
        }
    }

    // Read the value in memory pointed by PC and advances it
    fn fetch(&mut self) -> u8 {
        let data = self.mem.peek_at(self.cpu.pc);
        self.cpu.inc_pc();
        data
    }

    // Read the value in memory pointed by PC as an i16 and advances it
    fn fetch_16(&mut self) -> u16 {
        let pc = self.cpu.pc;
        self.cpu.pc = wrap_add!(self.cpu.pc, 2);
        self.mem.peek_at_16(pc)
    }

    // Address getters
    fn immediate(&mut self) -> u16 {
        let pc = self.cpu.pc;
        self.cpu.inc_pc();
        pc
    }

    fn zero_page(&mut self) -> u16 { self.fetch().into() }

    fn zero_page_x(&mut self) -> u16 { wrap_add!(self.fetch(), self.cpu.x).into() }

    fn zero_page_y(&mut self) -> u16 { wrap_add!(self.fetch(), self.cpu.y).into() }

    fn absolute(&mut self) -> u16 { self.fetch_16() }

    fn absolute_x(&mut self) -> u16 {
        let addr = self.fetch_16();
        wrap_add!(addr, self.cpu.x as u16)
    }

    fn absolute_y(&mut self) -> u16 {
        let addr = self.fetch_16();
        wrap_add!(addr, self.cpu.y as u16)
    }

    fn indirect(&mut self) -> u16 {
        let mut addr = self.fetch_16();
        addr = self.mem.peek_at_16(addr);
        self.mem.peek_at_16(addr)
    }

    fn indirect_x(&mut self) -> u16 {
        let addr = wrap_add!(self.fetch(), self.cpu.x) as u16;
        self.mem.peek_at_16(addr)
    }

    fn indirect_y(&mut self) -> u16 {
        let addr = self.fetch();
        let addr = self.mem.peek_at_16(addr.into());
        wrap_add!(addr, self.cpu.y as u16)
    }

    // Pushes a value onto the stack
    fn push(&mut self, value: u8) {
        let addr = self.cpu.sp;
        self.mem.put_at(addr.into(), value);
        self.cpu.sp -= 1;
    }

    // Pull a value from the stack
    fn pull(&mut self) -> u8 {
        self.cpu.sp += 1;
        let value = self.mem.peek_at(self.cpu.sp.into());
        value
    }

    // Bit test
    fn bit_test(&mut self, addr: u16) {
        let value = self.mem.peek_at(addr);
        self.cpu.p.change_zero(self.cpu.a & value);
        self.cpu.p.copy(Flags::Negative | Flags::Overflow, value);
    }

    // Addition
    fn addition(&mut self, addr: u16) {
        let value = self.mem.peek_at(addr);
        self.cpu.adc_a(value);
    }

    // Subtraction
    fn subtraction(&mut self, addr: u16) {
        let value = self.mem.peek_at(addr);
        // Since you should subtract (1 - carry) inverting the value
        // has the same effect as negating after the carry is added
        self.cpu.adc_a(!value);
        // Carry result is opposite to adding
        self.cpu.p.toggle(Flags::Carry);
    }

    // Executes one step of the CPU
    pub fn step(&mut self) {
        // Pipe
        macro_rules! pipe {
            ( $initial:expr $( => $s:ident $( .$ident:ident )* )* ) => {{
                let res = $initial;
                $( let res = $s $( .$ident )* (res); )*
                res.into()
            }}
        }

        // Set a value at an address
        macro_rules! set_mem {
            ( $setter:ident, $getter:ident ) => {{
                let addr = self.$setter();
                let value = self.cpu.$getter();
                self.mem.put_at(addr, value);
            }};
        }

        // Load macros
        macro_rules! lda { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.set_a) }; }
        macro_rules! ldx { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.set_x) }; }
        macro_rules! ldy { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.set_y) }; }

        // Logic operator macros
        macro_rules! and { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.a.bitand => self.cpu.set_a) }; }
        macro_rules! ora { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.a.bitor => self.cpu.set_a) }; }
        macro_rules! eor { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.a.bitxor => self.cpu.set_a) }; }

        // Compares
        macro_rules! cmp { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.cmp_a) }; }
        macro_rules! cpx { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.cmp_x) }; }
        macro_rules! cpy { ( $addr:ident ) => { pipe!(self.$addr() => self.mem.peek_at => self.cpu.cmp_y) }; }

        // Shift left
        macro_rules! asl {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.mem.peek_at(addr);
                self.cpu.p.change_left_shift(value);
                self.mem.put_at(addr, value << 1);
            }}
        }

        // Shift Right
        macro_rules! lsr {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.mem.peek_at(addr);
                self.cpu.p.change_right_shift(value);
                self.mem.put_at(addr, value >> 1);
            }}
        }

        // Rotates left
        macro_rules! rol {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.mem.peek_at(addr);
                self.cpu.p.change_left_shift(value);
                let carry = self.cpu.p.bits() & Flags::Carry.bits();
                self.mem.put_at(addr, (value << 1) | carry);
            }}
        }

        // Rotates right
        macro_rules! ror {
            ( $addr:ident ) => {{
                let addr = self.$addr();
                let mut value = self.mem.peek_at(addr);
                self.cpu.p.change_left_shift(value);
                let carry = (self.cpu.p.bits() & Flags::Carry.bits()) << 7;
                self.mem.put_at(addr, (value >> 1) | carry);
            }}
        }

        // Increment a register
        macro_rules! inc_mem {
            ( $addr:ident, $step:expr ) => {{
                let addr = self.$addr();
                let value = wrap_add!(self.mem.peek_at(addr), $step as u8);
                self.cpu.p.change_zero_and_negative(value);
                self.mem.put_at(addr, value);
            }}
        }

        // Branch if the condition is true
        macro_rules! branch {
            ( $condition:expr ) => {{
                let offset = (self.fetch() as i8) as u16;
                if $condition { self.cpu.pc = wrap_add!(self.cpu.pc, offset); }
            }}
        }

        let opcode = self.fetch();
        match opcode {
            // No operation
            opc::Nop => {}

            // Load into A
            opc::Lda::Immediate => lda!(immediate),
            opc::Lda::ZeroPage => lda!(zero_page),
            opc::Lda::ZeroPageX => lda!(zero_page_x),
            opc::Lda::Absolute => lda!(absolute),
            opc::Lda::AbsoluteX => lda!(absolute_x),
            opc::Lda::AbsoluteY => lda!(absolute_y),
            opc::Lda::IndirectX => lda!(indirect_x),
            opc::Lda::IndirectY => lda!(indirect_y),

            // Load into X
            opc::Ldx::Immediate => ldx!(immediate),
            opc::Ldx::ZeroPage => ldx!(zero_page),
            opc::Ldx::ZeroPageY => ldx!(zero_page_y),
            opc::Ldx::Absolute => ldx!(absolute),
            opc::Ldx::AbsoluteY => ldx!(absolute_y),

            // Load into Y
            opc::Ldy::Immediate => ldy!(immediate),
            opc::Ldy::ZeroPage => ldy!(zero_page),
            opc::Ldy::ZeroPageX => ldy!(zero_page_x),
            opc::Ldy::Absolute => ldy!(absolute),
            opc::Ldy::AbsoluteX => ldy!(absolute_x),

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
            opc::Tax => pipe!(self.cpu.get_a() => self.cpu.set_x),
            opc::Tay => pipe!(self.cpu.get_a() => self.cpu.set_y),
            opc::Txa => pipe!(self.cpu.get_x() => self.cpu.set_a),
            opc::Tya => pipe!(self.cpu.get_y() => self.cpu.set_a),
            opc::Tsx => pipe!(self.cpu.get_sp() => self.cpu.set_x),
            opc::Txs => pipe!(self.cpu.get_x() => self.cpu.set_sp),

            // Stack
            opc::Pha => pipe!(self.cpu.get_a() => self.push),
            opc::Php => pipe![self.cpu.get_p() | Flags::BreakCommand.bits() | Flags::Unused.bits() => self.push],
            opc::Pla => pipe!(self.pull() => self.cpu.set_a),
            opc::Plp => {
                let mut res = Flags::from(self.pull());
                res.copy(Flags::BreakCommand | Flags::Unused, self.cpu.get_p());
                self.cpu.set_p(res.bits());
            }

            // And
            opc::And::Immediate => and!(immediate),
            opc::And::ZeroPage => and!(zero_page),
            opc::And::ZeroPageX => and!(zero_page_x),
            opc::And::Absolute => and!(absolute),
            opc::And::AbsoluteX => and!(absolute_x),
            opc::And::AbsoluteY => and!(absolute_y),
            opc::And::IndirectX => and!(indirect_x),
            opc::And::IndirectY => and!(indirect_y),

            // Or
            opc::Ora::Immediate => ora!(immediate),
            opc::Ora::ZeroPage => ora!(zero_page),
            opc::Ora::ZeroPageX => ora!(zero_page_x),
            opc::Ora::Absolute => ora!(absolute),
            opc::Ora::AbsoluteX => ora!(absolute_x),
            opc::Ora::AbsoluteY => ora!(absolute_y),
            opc::Ora::IndirectX => ora!(indirect_x),
            opc::Ora::IndirectY => ora!(indirect_y),

            // Xor
            opc::Eor::Immediate => eor!(immediate),
            opc::Eor::ZeroPage => eor!(zero_page),
            opc::Eor::ZeroPageX => eor!(zero_page_x),
            opc::Eor::Absolute => eor!(absolute),
            opc::Eor::AbsoluteX => eor!(absolute_x),
            opc::Eor::AbsoluteY => eor!(absolute_y),
            opc::Eor::IndirectX => eor!(indirect_x),
            opc::Eor::IndirectY => eor!(indirect_y),

            // Bit test
            opc::Bit::ZeroPage => pipe!(self.zero_page() => self.bit_test),
            opc::Bit::Absolute => pipe!(self.absolute() => self.bit_test),

            // Addition
            opc::Adc::Immediate => pipe!(self.immediate() => self.addition),
            opc::Adc::ZeroPage => pipe!(self.zero_page() => self.addition),
            opc::Adc::ZeroPageX => pipe!(self.zero_page_x() => self.addition),
            opc::Adc::Absolute => pipe!(self.absolute() => self.addition),
            opc::Adc::AbsoluteX => pipe!(self.absolute_x() => self.addition),
            opc::Adc::AbsoluteY => pipe!(self.absolute_y() => self.addition),
            opc::Adc::IndirectX => pipe!(self.indirect_x() => self.addition),
            opc::Adc::IndirectY => pipe!(self.indirect_y() => self.addition),

            // Subtraction
            opc::Sbc::Immediate => pipe!(self.immediate() => self.subtraction),
            opc::Sbc::ZeroPage => pipe!(self.zero_page() => self.subtraction),
            opc::Sbc::ZeroPageX => pipe!(self.zero_page_x() => self.subtraction),
            opc::Sbc::Absolute => pipe!(self.absolute() => self.subtraction),
            opc::Sbc::AbsoluteX => pipe!(self.absolute_x() => self.subtraction),
            opc::Sbc::AbsoluteY => pipe!(self.absolute_y() => self.subtraction),
            opc::Sbc::IndirectX => pipe!(self.indirect_x() => self.subtraction),
            opc::Sbc::IndirectY => pipe!(self.indirect_y() => self.subtraction),

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
            opc::Inx => pipe!(wrap_add!(self.cpu.get_x(), 1i8 as u8) => self.cpu.set_x),
            opc::Dex => pipe!(wrap_add!(self.cpu.get_x(), -1i8 as u8) => self.cpu.set_x),
            opc::Iny => pipe!(wrap_add!(self.cpu.get_y(), 1i8 as u8) => self.cpu.set_y),
            opc::Dey => pipe!(wrap_add!(self.cpu.get_y(), -1i8 as u8) => self.cpu.set_y),

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
            opc::Cmp::Immediate => cmp!(immediate),
            opc::Cmp::ZeroPage => cmp!(zero_page),
            opc::Cmp::ZeroPageX => cmp!(zero_page_x),
            opc::Cmp::Absolute => cmp!(absolute),
            opc::Cmp::AbsoluteX => cmp!(absolute_x),
            opc::Cmp::AbsoluteY => cmp!(absolute_y),
            opc::Cmp::IndirectX => cmp!(indirect_x),
            opc::Cmp::IndirectY => cmp!(indirect_y),

            // Compare X
            opc::Cpx::Immediate => cpx!(immediate),
            opc::Cpx::ZeroPage => cpx!(zero_page),
            opc::Cpx::Absolute => cpx!(absolute),

            // Compare X
            opc::Cpy::Immediate => cpy!(immediate),
            opc::Cpy::ZeroPage => cpy!(zero_page),
            opc::Cpy::Absolute => cpy!(absolute),

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
            opc::Bvs => branch!(self.cpu.p.contains(Flags::Overflow)),
            opc::Bvc => branch!(!self.cpu.p.contains(Flags::Overflow)),

            // Jump
            opc::Jmp::Absolute => pipe!(self.absolute() => self.cpu.set_pc),
            opc::Jmp::Indirect => pipe!(self.indirect() => self.cpu.set_pc),

            // Call
            opc::Jsr => {
                let pc = wrap_add!(self.cpu.pc, 1);
                self.cpu.pc = self.fetch_16();

                self.mem.put_at_16(self.cpu.sp as u16, pc);
                self.cpu.sp = wrap_add!(self.cpu.sp, -2i8 as u8);
            }

            // Return
            opc::Rts => {
                self.cpu.sp = wrap_add!(self.cpu.sp, 2u8);
                self.cpu.pc = wrap_add!(self.mem.peek_at_16(self.cpu.sp as u16), 1);
            }

            // Not implemented
            _ => panic!("Opcode not implemented: 0x{:02x?}", opcode)
        }
    }
}
