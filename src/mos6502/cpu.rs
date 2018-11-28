use crate::mos6502::bus::Bus;
use crate::mos6502::cycle;
use crate::mos6502::microcode;
use crate::mos6502::reg::Reg;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct Cpu {
    // Registers
    pub reg: Reg,

    // Address Bus
    pub addr_bus: Bus,

    // Number of cycles since the CPU has been turned on.
    clock: u32,
}

impl Cpu {
    pub fn new(addr_bus: Bus) -> Self {
        Self { reg: Reg::new(), clock: 0, addr_bus }
    }

    pub fn get_clock(&self) -> u32 { self.clock }

    // Step a cycle
    pub fn step(&mut self) {
        self.clock += 1;

        // Last cycle fetches the opcode.
        if self.reg.get_cycle() == cycle::LAST {
            self.reg.fetch_opcode(&mut self.addr_bus);
            self.reg.set_next_cycle();
            return;
        }

        // Run an opcode
        macro_rules! run {
            ($opcode:ident) => {
                microcode::$opcode::run(&mut self.reg, & mut self.addr_bus);
            };
            ($opcode:ident, $mode:ident) => {
                microcode::$opcode::$mode::run(&mut self.reg, & mut self.addr_bus);
            };
        }

        match self.reg.get_current_instr() {
            0x00 => unimplemented!(),         /*bytes: 0 cycles: 7  _____=>_____ __      Brk, Implied   */
            0x01 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>____P R_ izx  Ora, IndirectX */
            0x02 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x03 => unimplemented!(),         /*bytes: 2 cycles: 8  A____=>____P RW izx  Slo, IndirectX */
            0x04 => run!(nop, zero_page),     /* bytes: 2 cycles: 3  _____=>_____ R_ zp   Nop, ZeroPage  */
            0x05 => unimplemented!(),         /*bytes: 2 cycles: 3  A____=>A___P R_ zp   Ora, ZeroPage  */
            0x06 => unimplemented!(),         /*bytes: 2 cycles: 5  _____=>____P RW zp   Asl, ZeroPage  */
            0x07 => unimplemented!(),         /*bytes: 2 cycles: 5  A____=>A___P RW zp   Slo, ZeroPage  */
            0x08 => unimplemented!(),         /*bytes: 1 cycles: 3  ___SP=>___S_ _W      Php, Implied   */
            0x09 => unimplemented!(),         /*bytes: 2 cycles: 2  _____=>A___P __      Ora, Immediate */
            0x0A => unimplemented!(),         /*bytes: 1 cycles: 2  A____=>A___P __      Asl, Implied   */
            0x0B => unimplemented!(),         /*bytes: 2 cycles: 2  A____=>____P __      Anc, Immediate */
            0x0C => run!(nop, absolute),      /* bytes: 3 cycles: 4  _____=>_____ R_ abs  Nop, Absolute  */
            0x0D => unimplemented!(),         /*bytes: 3 cycles: 4  A____=>A___P R_ abs  Ora, Absolute  */
            0x0E => unimplemented!(),         /*bytes: 3 cycles: 6  _____=>____P RW abs  Asl, Absolute  */
            0x0F => unimplemented!(),         /*bytes: 3 cycles: 6  A____=>A___P RW abs  Slo, Absolute  */
            0x10 => unimplemented!(),         /*bytes: 2 cycles: 2* ____P=>_____ __      Bpl, Relative  */
            0x11 => unimplemented!(),         /*bytes: 2 cycles: 5* A____=>____P R_ izy  Ora, IndirectY */
            0x12 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x13 => unimplemented!(),         /*bytes: 2 cycles: 8  A____=>____P RW izy  Slo, IndirectY */
            0x14 => run!(nop, zero_page_x),   /* bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX */
            0x15 => unimplemented!(),         /*bytes: 2 cycles: 4  A____=>A___P R_ zpx  Ora, ZeroPageX */
            0x16 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>____P RW zpx  Asl, ZeroPageX */
            0x17 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>A___P RW zpx  Slo, ZeroPageX */
            0x18 => unimplemented!(),         /*bytes: 1 cycles: 2  _____=>____P __      Clc, Implied   */
            0x19 => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>A___P R_ absy Ora, AbsoluteY */
            0x1A => run!(nop, implied),       /* bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied   */
            0x1B => unimplemented!(),         /*bytes: 3 cycles: 7  A____=>A___P RW absy Slo, AbsoluteY */
            0x1C => run!(nop, absolute_x),    /* bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX */
            0x1D => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>A___P R_ absx Ora, AbsoluteX */
            0x1E => unimplemented!(),         /*bytes: 3 cycles: 7  _____=>____P RW absx Asl, AbsoluteX */
            0x1F => unimplemented!(),         /*bytes: 3 cycles: 7  A____=>A___P RW absx Slo, AbsoluteX */
            0x20 => unimplemented!(),         /*bytes: X cycles: 6  ___S_=>___S_ _W      Jsr, Absolute  */
            0x21 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>A___P R_ izx  And, IndirectX */
            0x22 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x23 => unimplemented!(),         /*bytes: 2 cycles: 8  ____P=>A___P RW izx  Rla, IndirectX */
            0x24 => unimplemented!(),         /*bytes: 2 cycles: 3  A____=>____P R_ zp   Bit, ZeroPage  */
            0x25 => unimplemented!(),         /*bytes: 2 cycles: 3  A____=>A___P R_ zp   And, ZeroPage  */
            0x26 => unimplemented!(),         /*bytes: 2 cycles: 5  ____P=>____P RW zp   Rol, ZeroPage  */
            0x27 => unimplemented!(),         /*bytes: 2 cycles: 5  A___P=>A___P RW zp   Rla, ZeroPage  */
            0x28 => unimplemented!(),         /*bytes: 1 cycles: 4  ___S_=>___SP __      Plp, Implied   */
            0x29 => unimplemented!(),         /*bytes: 2 cycles: 2  A____=>A___P __      And, Immediate */
            0x2A => unimplemented!(),         /*bytes: 1 cycles: 2  A___P=>A___P __      Rol, ZeroPage  */
            0x2B => unimplemented!(),         /*bytes: 2 cycles: 2  A____=>____P __      Anc, ZeroPage  */
            0x2C => unimplemented!(),         /*bytes: 3 cycles: 4  A____=>____P R_ abs  Bit, Absolute  */
            0x2D => unimplemented!(),         /*bytes: 3 cycles: 4  A____=>A___P R_ abs  And, Absolute  */
            0x2E => unimplemented!(),         /*bytes: 3 cycles: 6  ____P=>____P RW abs  Rol, Absolute  */
            0x2F => unimplemented!(),         /*bytes: 3 cycles: 6  A___P=>A___P RW abs  Rla, Absolute  */
            0x30 => unimplemented!(),         /*bytes: 2 cycles: 2* _____=>_____ __      Bmi, Relative  */
            0x31 => unimplemented!(),         /*bytes: 2 cycles: 5* _____=>A___P R_ izy  And, IndirectY */
            0x32 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x33 => unimplemented!(),         /*bytes: 2 cycles: 8  ____P=>A___P RW izy  Rla, IndirectY */
            0x34 => run!(nop, zero_page_x),   /* bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX */
            0x35 => unimplemented!(),         /*bytes: 2 cycles: 4  A____=>A___P R_ zpx  And, ZeroPageX */
            0x36 => unimplemented!(),         /*bytes: 2 cycles: 6  ____P=>____P RW zpx  Rol, ZeroPageX */
            0x37 => unimplemented!(),         /*bytes: 2 cycles: 6  A___P=>A___P RW zpx  Rla, ZeroPageX */
            0x38 => unimplemented!(),         /*bytes: 1 cycles: 2  _____=>____P __      Sec, Implied   */
            0x39 => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>A___P R_ absy And, AbsoluteY */
            0x3A => run!(nop, implied),       /* bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied   */
            0x3B => unimplemented!(),         /*bytes: 3 cycles: 7  A___P=>A___P RW absy Rla, AbsoluteY */
            0x3C => run!(nop, absolute_x),    /* bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX */
            0x3D => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>A___P R_ absx And, AbsoluteX */
            0x3E => unimplemented!(),         /*bytes: 3 cycles: 7  ____P=>____P RW absx Rol, AbsoluteX */
            0x3F => unimplemented!(),         /*bytes: 3 cycles: 7  A___P=>A___P RW absx Rla, AbsoluteX */
            0x40 => unimplemented!(),         /*bytes: X cycles: 6  ___S_=>___SP __      Rti, Implied   */
            0x41 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>____P R_ izx  Eor, IndirectX */
            0x42 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x43 => unimplemented!(),         /*bytes: 2 cycles: 8  A____=>____P RW izx  Sre, IndirectX */
            0x44 => run!(nop, zero_page),     /* bytes: 2 cycles: 3  _____=>_____ R_ zp   Nop, ZeroPage  */
            0x45 => unimplemented!(),         /*bytes: 2 cycles: 3  A____=>A___P R_ zp   Eor, ZeroPage  */
            0x46 => unimplemented!(),         /*bytes: 2 cycles: 5  _____=>____P RW zp   Lsr, ZeroPage  */
            0x47 => unimplemented!(),         /*bytes: 2 cycles: 5  A____=>A___P RW zp   Sre, ZeroPage  */
            0x48 => unimplemented!(),         /*bytes: 1 cycles: 3  A__S_=>___S_ _W      Pha, Implied   */
            0x49 => unimplemented!(),         /*bytes: 2 cycles: 2  A____=>A___P __      Eor, Immediate */
            0x4A => unimplemented!(),         /*bytes: 1 cycles: 2  A____=>A___P __      Lsr, Implied   */
            0x4B => unimplemented!(),         /*bytes: 2 cycles: 2  A____=>A___P __      Alr, Immediate */
            0x4C => unimplemented!(),         /*bytes: X cycles: 3  _____=>_____ __      Jmp, Absolute  */
            0x4D => unimplemented!(),         /*bytes: 3 cycles: 4  A____=>A___P R_ abs  Eor, Absolute  */
            0x4E => unimplemented!(),         /*bytes: 3 cycles: 6  _____=>____P RW abs  Lsr, Absolute  */
            0x4F => unimplemented!(),         /*bytes: 3 cycles: 6  A____=>A___P RW abs  Sre, Absolute  */
            0x50 => unimplemented!(),         /*bytes: 2 cycles: 2* ____P=>_____ __      Bvc, Relative  */
            0x51 => unimplemented!(),         /*bytes: 2 cycles: 5* A____=>____P R_ izy  Eor, IndirectY */
            0x52 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x53 => unimplemented!(),         /*bytes: 2 cycles: 8  A____=>____P RW izy  Sre, IndirectY */
            0x54 => run!(nop, zero_page_x),   /* bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX */
            0x55 => unimplemented!(),         /*bytes: 2 cycles: 4  A____=>A___P R_ zpx  Eor, ZeroPageX */
            0x56 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>____P RW zpx  Lsr, ZeroPageX */
            0x57 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>A___P RW zpx  Sre, ZeroPageX */
            0x58 => unimplemented!(),         /*bytes: 1 cycles: 2  _____=>____P __      Cli, Implied   */
            0x59 => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>A___P R_ absy Eor, AbsoluteY */
            0x5A => run!(nop, implied),       /* bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied   */
            0x5B => unimplemented!(),         /*bytes: 3 cycles: 7  A____=>A___P RW absy Sre, AbsoluteY */
            0x5C => run!(nop, absolute_x),    /* bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX */
            0x5D => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>A___P R_ absx Eor, AbsoluteX */
            0x5E => unimplemented!(),         /*bytes: 3 cycles: 7  _____=>____P RW absx Lsr, AbsoluteX */
            0x5F => unimplemented!(),         /*bytes: 3 cycles: 7  A____=>A___P RW absx Sre, AbsoluteX */
            0x60 => unimplemented!(),         /*bytes: X cycles: 6  ___S_=>___S_ __      Rts, Implied   */
            0x61 => unimplemented!(),         /*bytes: 2 cycles: 6  A___P=>A___P R_ izx  Adc, IndirectX */
            0x62 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x63 => unimplemented!(),         /*bytes: 2 cycles: 8  A___P=>A___P RW izx  Rra, IndirectX */
            0x64 => run!(nop, zero_page),     /* bytes: 2 cycles: 3  _____=>_____ R_ zp   Nop, ZeroPage  */
            0x65 => unimplemented!(),         /*bytes: 2 cycles: 3  A___P=>A___P R_ zp   Adc, ZeroPage  */
            0x66 => unimplemented!(),         /*bytes: 2 cycles: 5  ____P=>____P RW zp   Ror, ZeroPage  */
            0x67 => unimplemented!(),         /*bytes: 2 cycles: 5  A___P=>A___P RW zp   Rra, ZeroPage  */
            0x68 => unimplemented!(),         /*bytes: 1 cycles: 4  ___S_=>A__SP __      Pla, Implied   */
            0x69 => unimplemented!(),         /*bytes: 2 cycles: 2  A___P=>A___P __      Adc, Immediate */
            0x6A => unimplemented!(),         /*bytes: 1 cycles: 2  A___P=>A___P __      Ror, Implied   */
            0x6B => unimplemented!(),         /*bytes: 2 cycles: 2  A___P=>A___P __      Arr, Immediate */
            0x6C => unimplemented!(),         /*bytes: X cycles: 5  _____=>_____ __      Jmp, Indirect  */
            0x6D => unimplemented!(),         /*bytes: 3 cycles: 4  A___P=>A___P R_ abs  Adc, Absolute  */
            0x6E => unimplemented!(),         /*bytes: 3 cycles: 6  ____P=>____P RW abs  Ror, Absolute  */
            0x6F => unimplemented!(),         /*bytes: 3 cycles: 6  A___P=>A___P RW abs  Rra, Absolute  */
            0x70 => unimplemented!(),         /*bytes: 2 cycles: 2* _____=>_____ __      Bvs, Relative  */
            0x71 => unimplemented!(),         /*bytes: 2 cycles: 5* A___P=>A___P R_ izy  Adc, IndirectY */
            0x72 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x73 => unimplemented!(),         /*bytes: 2 cycles: 8  A___P=>A___P RW izy  Rra, IndirectY */
            0x74 => run!(nop, zero_page_x),   /* bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX */
            0x75 => unimplemented!(),         /*bytes: 2 cycles: 4  A___P=>A___P R_ zpx  Adc, ZeroPageX */
            0x76 => unimplemented!(),         /*bytes: 2 cycles: 6  ____P=>____P RW zpx  Ror, ZeroPageX */
            0x77 => unimplemented!(),         /*bytes: 2 cycles: 6  A___P=>A___P RW zpx  Rra, ZeroPageX */
            0x78 => unimplemented!(),         /*bytes: 1 cycles: 2  _____=>____P __      Sei, Implied   */
            0x79 => unimplemented!(),         /*bytes: 3 cycles: 4* A___P=>A___P R_ absy Adc, AbsoluteY */
            0x7A => run!(nop, implied),       /* bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied   */
            0x7B => unimplemented!(),         /*bytes: 3 cycles: 7  A___P=>A___P RW absy Rra, AbsoluteY */
            0x7C => run!(nop, absolute_x),    /* bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX */
            0x7D => unimplemented!(),         /*bytes: 3 cycles: 4* A___P=>A___P R_ absx Adc, AbsoluteX */
            0x7E => unimplemented!(),         /*bytes: 3 cycles: 7  ____P=>____P RW absx Ror, AbsoluteX */
            0x7F => unimplemented!(),         /*bytes: 3 cycles: 7  A___P=>A___P RW absx Rra, AbsoluteX */
            0x80 => run!(nop, immediate),     /* bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate */
            0x81 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>_____ RW izx  Sta, IndirectX */
            0x82 => run!(nop, immediate),     /* bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate */
            0x83 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>_____ RW izx  Sax, IndirectX */
            0x84 => unimplemented!(),         /*bytes: 2 cycles: 3  __Y__=>_____ _W zp   Sty, ZeroPage  */
            0x85 => unimplemented!(),         /*bytes: 2 cycles: 3  A____=>_____ _W zp   Sta, ZeroPage  */
            0x86 => unimplemented!(),         /*bytes: 2 cycles: 3  _X___=>_____ _W zp   Stx, ZeroPage  */
            0x87 => unimplemented!(),         /*bytes: 2 cycles: 3  _____=>_____ _W zp   Sax, ZeroPage  */
            0x88 => unimplemented!(),         /*bytes: 1 cycles: 2  __Y__=>__Y_P __      Dey, Implied   */
            0x89 => run!(nop, immediate),     /* bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate */
            0x8A => unimplemented!(),         /*bytes: 1 cycles: 2  _X___=>A___P __      Txa, Implied   */
            0x8B => unimplemented!(),         /*bytes: 2 cycles: 2  _____=>A___P __      Xaa, Immediate */
            0x8C => unimplemented!(),         /*bytes: 3 cycles: 4  __Y__=>_____ _W abs  Sty, Absolute  */
            0x8D => unimplemented!(),         /*bytes: 3 cycles: 4  A____=>_____ _W abs  Sta, Absolute  */
            0x8E => unimplemented!(),         /*bytes: 3 cycles: 4  _X___=>_____ _W abs  Stx, Absolute  */
            0x8F => unimplemented!(),         /*bytes: 3 cycles: 4  _____=>_____ _W abs  Sax, Absolute  */
            0x90 => unimplemented!(),         /*bytes: 2 cycles: 2* ____P=>_____ __      Bcc, Relative  */
            0x91 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>_____ RW izy  Sta, IndirectY */
            0x92 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0x93 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>_____ RW izy  Ahx, IndirectY */
            0x94 => unimplemented!(),         /*bytes: 2 cycles: 4  __Y__=>_____ RW zpx  Sty, ZeroPageX */
            0x95 => unimplemented!(),         /*bytes: 2 cycles: 4  A____=>_____ RW zpx  Sta, ZeroPageX */
            0x96 => unimplemented!(),         /*bytes: 2 cycles: 4  _X___=>_____ RW zpy  Stx, ZeroPageY */
            0x97 => unimplemented!(),         /*bytes: 2 cycles: 4  _____=>_____ RW zpy  Sax, ZeroPageY */
            0x98 => unimplemented!(),         /*bytes: 1 cycles: 2  __Y__=>A___P __      Tya, Implied   */
            0x99 => unimplemented!(),         /*bytes: 3 cycles: 5  A____=>_____ RW absy Sta, AbsoluteY */
            0x9A => unimplemented!(),         /*bytes: X cycles: 2  _X___=>___S_ __      Txs, Implied   */
            0x9B => unimplemented!(),         /*bytes: X cycles: 5  __Y__=>___S_ _W      Tas, AbsoluteY */
            0x9C => unimplemented!(),         /*bytes: 3 cycles: 5  __Y__=>_____ RW absx Shy, AbsoluteX */
            0x9D => unimplemented!(),         /*bytes: 3 cycles: 5  A____=>_____ RW absx Sta, AbsoluteX */
            0x9E => unimplemented!(),         /*bytes: 3 cycles: 5  _X___=>_____ RW absy Shx, AbsoluteY */
            0x9F => unimplemented!(),         /*bytes: 3 cycles: 5  _____=>_____ RW absy Ahx, AbsoluteY */
            0xA0 => unimplemented!(),         /*bytes: 2 cycles: 2  _____=>__Y_P __      Ldy, Immediate */
            0xA1 => run!(lda, indirect_x),    /*bytes: 2 cycles: 6  _____=>A___P R_ izx  Lda, IndirectX */
            0xA2 => unimplemented!(),         /*bytes: 2 cycles: 2  _____=>_X__P __      Ldx, Immediate */
            0xA3 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>AX__P R_ izx  Lax, IndirectX */
            0xA4 => unimplemented!(),         /*bytes: 2 cycles: 3  _____=>__Y_P R_ zp   Ldy, ZeroPage  */
            0xA5 => run!(lda, zero_page),     /*bytes: 2 cycles: 3  _____=>A___P R_ zp   Lda, ZeroPage  */
            0xA6 => unimplemented!(),         /*bytes: 2 cycles: 3  _____=>_X__P R_ zp   Ldx, ZeroPage  */
            0xA7 => unimplemented!(),         /*bytes: 2 cycles: 3  _____=>AX__P R_ zp   Lax, ZeroPage  */
            0xA8 => unimplemented!(),         /*bytes: 1 cycles: 2  A____=>__Y_P __      Tay, Implied   */
            0xA9 => run!(lda, immediate),     /*bytes: 2 cycles: 2  _____=>A___P __      Lda, Immediate */
            0xAA => unimplemented!(),         /*bytes: 1 cycles: 2  A____=>_X__P __      Tax, Implied   */
            0xAB => unimplemented!(),         /*bytes: 2 cycles: 2  A____=>AX__P __      Lax, Immediate */
            0xAC => unimplemented!(),         /*bytes: 3 cycles: 4  _____=>__Y_P R_ abs  Ldy, Absolute  */
            0xAD => run!(lda, absolute),      /*bytes: 3 cycles: 4  _____=>A___P R_ abs  Lda, Absolute  */
            0xAE => unimplemented!(),         /*bytes: 3 cycles: 4  _____=>_X__P R_ abs  Ldx, Absolute  */
            0xAF => unimplemented!(),         /*bytes: 3 cycles: 4  _____=>AX__P R_ abs  Lax, Absolute  */
            0xB0 => unimplemented!(),         /*bytes: 2 cycles: 2* _____=>_____ __      Bcs, Relative  */
            0xB1 => unimplemented!(),         /*bytes: 2 cycles: 5* _____=>A___P R_ izy  Lda, IndirectY */
            0xB2 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0xB3 => unimplemented!(),         /*bytes: 2 cycles: 5* _____=>AX__P R_ izy  Lax, IndirectY */
            0xB4 => unimplemented!(),         /*bytes: 2 cycles: 4  _____=>__Y_P R_ zpx  Ldy, ZeroPageX */
            0xB5 => run!(lda, zero_page_x),   /*bytes: 2 cycles: 4  _____=>A___P R_ zpx  Lda, ZeroPageX */
            0xB6 => unimplemented!(),         /*bytes: 2 cycles: 4  _____=>_X__P R_ zpy  Ldx, ZeroPageY */
            0xB7 => unimplemented!(),         /*bytes: 2 cycles: 4  _____=>AX__P R_ zpy  Lax, ZeroPageY */
            0xB8 => unimplemented!(),         /*bytes: 1 cycles: 2  _____=>____P __      Clv, Implied   */
            0xB9 => run!(lda, absolute_y),    /*bytes: 3 cycles: 4* _____=>A___P R_ absy Lda, AbsoluteY */
            0xBA => unimplemented!(),         /*bytes: 1 cycles: 2  ___S_=>_X__P __      Tsx, Implied  */
            0xBB => unimplemented!(),         /*bytes: 3 cycles: 4* ___S_=>AX_SP R_ absy Las, AbsoluteY */
            0xBC => unimplemented!(),         /*bytes: 3 cycles: 4* _____=>__Y_P R_ absx Ldy, AbsoluteX */
            0xBD => run!(lda, absolute_x),    /*bytes: 3 cycles: 4* _____=>A___P R_ absx Lda, AbsoluteX */
            0xBE => unimplemented!(),         /*bytes: 3 cycles: 4* _____=>_X__P R_ absy Ldx, AbsoluteY */
            0xBF => unimplemented!(),         /*bytes: 3 cycles: 4* _____=>AX__P R_ absy Lax, AbsoluteY */
            0xC0 => unimplemented!(),         /*bytes: 2 cycles: 2  __Y__=>____P __      Cpy, Immediate */
            0xC1 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>____P R_ izx  Cmp, IndirectX */
            0xC2 => run!(nop, immediate),     /* bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate */
            0xC3 => unimplemented!(),         /*bytes: 2 cycles: 8  A____=>____P RW izx  Dcp, IndirectX */
            0xC4 => unimplemented!(),         /*bytes: 2 cycles: 3  __Y__=>____P R_ zp   Cpy, ZeroPage  */
            0xC5 => unimplemented!(),         /*bytes: 2 cycles: 3  A____=>____P R_ zp   Cmp, ZeroPage  */
            0xC6 => unimplemented!(),         /*bytes: 2 cycles: 5  _____=>____P RW zp   Dec, ZeroPage  */
            0xC7 => unimplemented!(),         /*bytes: 2 cycles: 5  A____=>____P RW zp   Dcp, ZeroPage  */
            0xC8 => unimplemented!(),         /*bytes: 1 cycles: 2  __Y__=>__Y_P __      Iny, Implied   */
            0xC9 => unimplemented!(),         /*bytes: 2 cycles: 2  A____=>____P __      Cmp, Immediate */
            0xCA => unimplemented!(),         /*bytes: 1 cycles: 2  _X___=>_X__P __      Dex, Implied   */
            0xCB => unimplemented!(),         /*bytes: 2 cycles: 2  _____=>_X__P __      Axs, Immediate */
            0xCC => unimplemented!(),         /*bytes: 3 cycles: 4  __Y__=>____P R_ abs  Cpy, Absolute  */
            0xCD => unimplemented!(),         /*bytes: 3 cycles: 4  A____=>____P R_ abs  Cmp, Absolute  */
            0xCE => unimplemented!(),         /*bytes: 3 cycles: 6  _____=>____P RW abs  Dec, Absolute  */
            0xCF => unimplemented!(),         /*bytes: 3 cycles: 6  A____=>____P RW abs  Dcp, Absolute  */
            0xD0 => unimplemented!(),         /*bytes: 2 cycles: 2* ____P=>_____ __      Bne, Relative  */
            0xD1 => unimplemented!(),         /*bytes: 2 cycles: 5* A____=>____P R_ izy  Cmp, IndirectY */
            0xD2 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0xD3 => unimplemented!(),         /*bytes: 2 cycles: 8  A____=>____P RW izy  Dcp, IndirectY */
            0xD4 => run!(nop, zero_page_x),   /* bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX */
            0xD5 => unimplemented!(),         /*bytes: 2 cycles: 4  A____=>____P R_ zpx  Cmp, ZeroPageX */
            0xD6 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>____P RW zpx  Dec, ZeroPageX */
            0xD7 => unimplemented!(),         /*bytes: 2 cycles: 6  A____=>____P RW zpx  Dcp, ZeroPageX */
            0xD8 => unimplemented!(),         /*bytes: 1 cycles: 2  _____=>____P __      Cld, Implied   */
            0xD9 => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>____P R_ absy Cmp, AbsoluteY */
            0xDA => run!(nop, implied),       /* bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied   */
            0xDB => unimplemented!(),         /*bytes: 3 cycles: 7  A____=>____P RW absy Dcp, AbsoluteY */
            0xDC => run!(nop, absolute_x),    /* bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX */
            0xDD => unimplemented!(),         /*bytes: 3 cycles: 4* A____=>____P R_ absx Cmp, AbsoluteX */
            0xDE => unimplemented!(),         /*bytes: 3 cycles: 7  _____=>____P RW absx Dec, AbsoluteX */
            0xDF => unimplemented!(),         /*bytes: 3 cycles: 7  A____=>____P RW absx Dcp, AbsoluteX */
            0xE0 => unimplemented!(),         /*bytes: 2 cycles: 2  _X___=>____P __      Cpx, Immediate */
            0xE1 => unimplemented!(),         /*bytes: 2 cycles: 6  A___P=>A___P R_ izx  Sbc, IndirectX */
            0xE2 => run!(nop, immediate),     /* bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate */
            0xE3 => unimplemented!(),         /*bytes: 2 cycles: 8  A___P=>A___P RW izx  Isc, IndirectX */
            0xE4 => unimplemented!(),         /*bytes: 2 cycles: 3  _X___=>____P R_ zp   Cpx, ZeroPage  */
            0xE5 => unimplemented!(),         /*bytes: 2 cycles: 3  A___P=>A___P R_ zp   Sbc, ZeroPage  */
            0xE6 => unimplemented!(),         /*bytes: 2 cycles: 5  _____=>____P RW zp   Inc, ZeroPage  */
            0xE7 => unimplemented!(),         /*bytes: 2 cycles: 5  A___P=>A___P RW zp   Isc, ZeroPage  */
            0xE8 => unimplemented!(),         /*bytes: 1 cycles: 2  _X___=>_X__P __      Inx, Implied   */
            0xE9 => unimplemented!(),         /*bytes: 2 cycles: 2  A___P=>A___P __      Sbc, Immediate */
            0xEA => run!(nop, implied),       /* bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied   */
            0xEB => unimplemented!(),         /*bytes: 2 cycles: 2  A___P=>A___P __      Sbc, Immediate */
            0xEC => unimplemented!(),         /*bytes: 3 cycles: 4  _X___=>____P R_ abs  Cpx, Absolute  */
            0xED => unimplemented!(),         /*bytes: 3 cycles: 4  A___P=>A___P R_ abs  Sbc, Absolute  */
            0xEE => unimplemented!(),         /*bytes: 3 cycles: 6  _____=>____P RW abs  Inc, Absolute  */
            0xEF => unimplemented!(),         /*bytes: 3 cycles: 6  A___P=>A___P RW abs  Isc, Absolute  */
            0xF0 => unimplemented!(),         /*bytes: 2 cycles: 2* _____=>_____ __      Beq, Relative  */
            0xF1 => unimplemented!(),         /*bytes: 2 cycles: 5* A___P=>A___P R_ izy  Sbc, IndirectY */
            0xF2 => run!(kil),                /* Crashes                                  Kil, Implied   */
            0xF3 => unimplemented!(),         /*bytes: 2 cycles: 8  A___P=>A___P RW izy  Isc, IndirectY */
            0xF4 => run!(nop, zero_page_x),   /* bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX */
            0xF5 => unimplemented!(),         /*bytes: 2 cycles: 4  A___P=>A___P R_ zpx  Sbc, ZeroPageX */
            0xF6 => unimplemented!(),         /*bytes: 2 cycles: 6  _____=>____P RW zpx  Inc, ZeroPageX */
            0xF7 => unimplemented!(),         /*bytes: 2 cycles: 6  A___P=>A___P RW zpx  Isc, ZeroPageX */
            0xF8 => unimplemented!(),         /*bytes: 1 cycles: 2  _____=>____P __      Sed, Implied   */
            0xF9 => unimplemented!(),         /*bytes: 3 cycles: 4* A___P=>A___P R_ absy Sbc, AbsoluteY */
            0xFA => run!(nop, implied),       /* bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied   */
            0xFB => unimplemented!(),         /*bytes: 3 cycles: 7  A___P=>A___P RW absy Isc, AbsoluteY */
            0xFC => run!(nop, absolute_x),   /* bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX */
            0xFD => unimplemented!(),         /*bytes: 3 cycles: 4* A___P=>A___P R_ absx Sbc, AbsoluteX */
            0xFE => unimplemented!(),         /*bytes: 3 cycles: 7  _____=>____P RW absx Inc, AbsoluteX */
            0xFF => unimplemented!(),         /*bytes: 3 cycles: 7  A___P=>A___P RW absx Isc, AbsoluteX */
            _ => unreachable!("Opcode not implemented: {:X}", self.reg.get_current_instr())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(bus: Vec<u8>, size: i16, clock: u32, setup: fn(&mut Cpu), result: fn(&mut Cpu)) {
        let bus = Bus::new(bus);

        let mut cpu = Cpu::new(bus);
        setup(&mut cpu);

        let mut check = cpu.clone();
        result(&mut check);

        check.clock = clock;
        check.reg.s_pc(cpu.reg.get_pc().wrapping_add(size as u16));

        loop {
            cpu.step();
            if cpu.reg.get_cycle() == cycle::LAST { break; }
            if cpu.clock > 100 { panic!("CPU couldn't finish."); }
        }

        check.reg.s_instr(cpu.reg.get_current_instr());
        check.reg.s_db(cpu.reg.get_data_bus());
        check.reg.s_ab(cpu.reg.get_addr_bus());
        check.reg.s_oper(cpu.reg.get_m());
        check.reg.s_oper_other(cpu.reg.get_n());
        check.reg.s_oper_v(cpu.reg.get_internal_overflow());

        assert_eq!(check, cpu);
    }

    fn as_is(_: &mut Cpu) {}

    #[test]
    #[should_panic(expected = "CPU couldn't finish.")]
    fn kil() { run(vec![0x02], 0, 0, as_is, as_is); }

    mod nop {
        use super::*;

        #[test]
        fn implied() { run(vec![0xea], 1, 2, as_is, as_is); }

        #[test]
        fn immediate() { run(vec![0xe2], 2, 2, as_is, as_is); }

        #[test]
        fn zero_page() { run(vec![0x64], 2, 3, as_is, as_is); }

        #[test]
        fn zero_page_x() { run(vec![0x14], 2, 4, as_is, as_is); }

        #[test]
        fn absolute() { run(vec![0x0C], 3, 4, as_is, as_is); }

        #[test]
        fn absolute_x() { run(vec![0x1C, 0xfe, 0x00], 3, 4, as_is, as_is); }

        #[test]
        fn absolute_x_cross_page() {
            run(vec![0x1C, 0xff, 0x00], 3, 5,
                |cpu| { cpu.reg.s_x(1) },
                as_is,
            );
        }
    }

    mod lda {
        use super::*;

        #[test]
        fn immediate() {
            run(vec![0xA9, 0x01], 2, 2, as_is,
                |cpu| { cpu.reg.s_a(0x01) },
            );
        }

        #[test]
        fn immediate_zero() {
            run(vec![0xA9, 0x00], 2, 2,
                |cpu| { cpu.reg.s_a(0x07); },
                |cpu| {
                    cpu.reg.s_z(true);
                    cpu.reg.s_a(0x00);
                },
            );
        }

        #[test]
        fn immediate_negative() {
            run(vec![0xA9, -0x01i8 as u8], 2, 2, as_is,
                |cpu| {
                    cpu.reg.s_n(true);
                    cpu.reg.s_a(-0x01i8 as u8);
                },
            );
        }

        #[test]
        fn zero_page() {
            run(vec![0xA5, 0x02, 0x09], 2, 3, as_is,
                |cpu| { cpu.reg.s_a(0x09) },
            );
        }

        #[test]
        fn zero_page_x() {
            run(vec![0xB5, 0x02, 0x08, 0x09], 2, 4,
                |cpu| { cpu.reg.s_x(0x01) },
                |cpu| { cpu.reg.s_a(0x09) },
            );
        }

        #[test]
        fn absolute() {
            run(vec![0xAD, 0x04, 0x00, 0x09, 0x10], 3, 4, as_is,
                |cpu| { cpu.reg.s_a(0x10) },
            );
        }

        #[test]
        fn absolute_x() {
            run(vec![0xBD, 0x04, 0x00, 0xff, 0x09, 0x10], 3, 4,
                |cpu| { cpu.reg.s_x(0x01) },
                |cpu| { cpu.reg.s_a(0x10) },
            );
        }

        #[test]
        fn absolute_x_cross_page() {
            run(vec![0xBD, 0xff, 0xff, 0xee, 0x09, 0x10], 3, 5,
                |cpu| { cpu.reg.s_x(0x06) },
                |cpu| { cpu.reg.s_a(0x10) },
            );
        }

        #[test]
        fn indirect_x() {
            run(vec![0xA1, 0x03, 0xff, 0xff, 0xff, 0x07, 0x00, 0x10, 0xff, 0xff, 0xff], 2, 6,
                |cpu| { cpu.reg.s_x(0x02) },
                |cpu| { cpu.reg.s_a(0x10) },
            );
        }
    }
}
