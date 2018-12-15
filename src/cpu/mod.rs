use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::bus::Bus;
use crate::cpu::log::Log;
use crate::cpu::reg::Reg;

pub mod cycle;
pub mod flags;
pub mod log;
pub mod opc;
pub mod reg;

pub struct Cpu {
    // Logger
    pub log: Log,

    // Registers
    pub reg: Reg,

    // Address Bus
    pub bus: Rc<RefCell<Bus>>,

    // Number of cycles since the CPU has been turned on.
    clock: u32,

    // Flags to indicate internal operations
    oam_transferring: bool,
    resetting: bool,
    interrupting: bool,
}

impl Cpu {
    pub fn new(bus: Rc<RefCell<Bus>>) -> Self {
        let mut res = Self {
            log: Log::new(),
            reg: Reg::new(),
            clock: 0,
            bus,
            oam_transferring: false,
            resetting: false,
            interrupting: false,
        };
        res.reset();
        res
    }

    pub fn get_clock(&self) -> u32 { self.clock }
    pub fn set_clock(&mut self, value: u32) { self.clock = value }

    // Step a cycle
    #[allow(clippy::cyclomatic_complexity)]
    pub fn step(&mut self) {
        trace!(target: "opcode", "T{}", self.reg.get_cycle());
        self.clock += 1;

        // Run an opcode
        macro_rules! run {
            ($code:ident) => {{
                self.$code();
                self.reg.set_next_cycle();
            }};

            ($code:ident, $mode:ident) => {{
                if let Some(res) = self.$mode() {
                    self.log.unofficial = false;
                    self.log.mnemonic = self.$code(res).0;
                }

                self.reg.set_next_cycle();
            }}
        }

        if self.reg.is_last_cycle() {
            // Last cycle fetches the opcode.
            let pc = self.fetch_pc();
            self.reg.set_current_instr(pc);
            self.reg.set_next_cycle();

            let bus = self.bus.borrow();
            if bus.reset {
                self.resetting = true
            } else if bus.nmi || (bus.irq && !self.reg.get_p().get_interrupt_disable()) {
                self.interrupting = true;
            } else if bus.ppu.oam_transfer {
                self.oam_transferring = true
            }

            return;
        }

        self.log.skip = true;

        // Reset subroutine. Will clear the flag when finished.
        if self.resetting { return run!(rst); }

        // If interrupted runt the BRK instruction.
        if self.interrupting { return run!(brk); }

        // Transfer OAM. Will clear the flag when finished.
        if self.oam_transferring { return run!(oam); }

        self.log.skip = false;

        #[allow(clippy::match_same_arms)]
            match self.reg.get_current_instr() {
            0x00 => run!(brk),                  /*bytes: 0 cycles: 7  _____=>_____ __      Brk, Implied     */
            0x01 => run!(ora, r_indirect_x),    /*bytes: 2 cycles: 6  A____=>____P R_ izx  Ora, IndirectX   */
            0x02 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x03 => run!(slo, rw_indirect_x),   /*bytes: 2 cycles: 8  A____=>____P RW izx  Slo, IndirectX   */
            0x04 => run!(dop, r_zero_page),     /*bytes: 2 cycles: 3  _____=>_____ R_ zp   Nop, ZeroPage    */
            0x05 => run!(ora, r_zero_page),     /*bytes: 2 cycles: 3  A____=>A___P R_ zp   Ora, ZeroPage    */
            0x06 => run!(asl, rw_zero_page),    /*bytes: 2 cycles: 5  _____=>____P RW zp   Asl, ZeroPage    */
            0x07 => run!(slo, rw_zero_page),    /*bytes: 2 cycles: 5  A____=>A___P RW zp   Slo, ZeroPage    */
            0x08 => run!(php, w_stack),         /*bytes: 1 cycles: 3  ___SP=>___S_ _W      Php, Implied     */
            0x09 => run!(ora, immediate),       /*bytes: 2 cycles: 2  _____=>A___P __      Ora, Immediate   */
            0x0A => run!(asl_acc, accumulator), /*bytes: 1 cycles: 2  A____=>A___P __      Asl, Accumulator */
            0x0B => run!(anc, immediate),       /*bytes: 2 cycles: 2  A____=>____P __      Anc, Immediate   */
            0x0C => run!(dop, r_absolute),      /*bytes: 3 cycles: 4  _____=>_____ R_ abs  Nop, Absolute    */
            0x0D => run!(ora, r_absolute),      /*bytes: 3 cycles: 4  A____=>A___P R_ abs  Ora, Absolute    */
            0x0E => run!(asl, rw_absolute),     /*bytes: 3 cycles: 6  _____=>____P RW abs  Asl, Absolute    */
            0x0F => run!(slo, rw_absolute),     /*bytes: 3 cycles: 6  A____=>A___P RW abs  Slo, Absolute    */
            0x10 => run!(bpl),                  /*bytes: 2 cycles: 2* ____P=>_____ __      Bpl, Relative    */
            0x11 => run!(ora, r_indirect_y),    /*bytes: 2 cycles: 5* A____=>____P R_ izy  Ora, IndirectY   */
            0x12 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x13 => run!(slo, rw_indirect_y),   /*bytes: 2 cycles: 8  A____=>____P RW izy  Slo, IndirectY   */
            0x14 => run!(dop, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX   */
            0x15 => run!(ora, r_zero_page_x),   /*bytes: 2 cycles: 4  A____=>A___P R_ zpx  Ora, ZeroPageX   */
            0x16 => run!(asl, rw_zero_page_x),  /*bytes: 2 cycles: 6  _____=>____P RW zpx  Asl, ZeroPageX   */
            0x17 => run!(slo, rw_zero_page_x),  /*bytes: 2 cycles: 6  A____=>A___P RW zpx  Slo, ZeroPageX   */
            0x18 => run!(clc, implied),         /*bytes: 1 cycles: 2  _____=>____P __      Clc, Implied     */
            0x19 => run!(ora, r_absolute_y),    /*bytes: 3 cycles: 4* A____=>A___P R_ absy Ora, AbsoluteY   */
            0x1A => run!(mop, implied),         /*bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied     */
            0x1B => run!(slo, rw_absolute_y),   /*bytes: 3 cycles: 7  A____=>A___P RW absy Slo, AbsoluteY   */
            0x1C => run!(dop, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX   */
            0x1D => run!(ora, r_absolute_x),    /*bytes: 3 cycles: 4* A____=>A___P R_ absx Ora, AbsoluteX   */
            0x1E => run!(asl, rw_absolute_x),   /*bytes: 3 cycles: 7  _____=>____P RW absx Asl, AbsoluteX   */
            0x1F => run!(slo, rw_absolute_x),   /*bytes: 3 cycles: 7  A____=>A___P RW absx Slo, AbsoluteX   */
            0x20 => run!(jsr),                  /*bytes: X cycles: 6  ___S_=>___S_ _W      Jsr, Absolute    */
            0x21 => run!(and, r_indirect_x),    /*bytes: 2 cycles: 6  _____=>A___P R_ izx  And, IndirectX   */
            0x22 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x23 => run!(rla, rw_indirect_x),   /*bytes: 2 cycles: 8  ____P=>A___P RW izx  Rla, IndirectX   */
            0x24 => run!(bit, r_zero_page),     /*bytes: 2 cycles: 3  A____=>____P R_ zp   Bit, ZeroPage    */
            0x25 => run!(and, r_zero_page),     /*bytes: 2 cycles: 3  A____=>A___P R_ zp   And, ZeroPage    */
            0x26 => run!(rol, rw_zero_page),    /*bytes: 2 cycles: 5  ____P=>____P RW zp   Rol, ZeroPage    */
            0x27 => run!(rla, rw_zero_page),    /*bytes: 2 cycles: 5  A___P=>A___P RW zp   Rla, ZeroPage    */
            0x28 => run!(plp, r_stack),         /*bytes: 1 cycles: 4  ___S_=>___SP __      Plp, Implied     */
            0x29 => run!(and, immediate),       /*bytes: 2 cycles: 2  A____=>A___P __      And, Immediate   */
            0x2A => run!(rol_acc, accumulator), /*bytes: 1 cycles: 2  A___P=>A___P __      Rol, Accumulator */
            0x2B => run!(anc, immediate),       /*bytes: 2 cycles: 2  A____=>____P __      Anc, Immediate   */
            0x2C => run!(bit, r_absolute),      /*bytes: 3 cycles: 4  A____=>____P R_ abs  Bit, Absolute    */
            0x2D => run!(and, r_absolute),      /*bytes: 3 cycles: 4  A____=>A___P R_ abs  And, Absolute    */
            0x2E => run!(rol, rw_absolute),     /*bytes: 3 cycles: 6  ____P=>____P RW abs  Rol, Absolute    */
            0x2F => run!(rla, rw_absolute),     /*bytes: 3 cycles: 6  A___P=>A___P RW abs  Rla, Absolute    */
            0x30 => run!(bmi),                  /*bytes: 2 cycles: 2* _____=>_____ __      Bmi, Relative    */
            0x31 => run!(and, r_indirect_y),    /*bytes: 2 cycles: 5* _____=>A___P R_ izy  And, IndirectY   */
            0x32 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x33 => run!(rla, rw_indirect_y),   /*bytes: 2 cycles: 8  ____P=>A___P RW izy  Rla, IndirectY   */
            0x34 => run!(dop, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX   */
            0x35 => run!(and, r_zero_page_x),   /*bytes: 2 cycles: 4  A____=>A___P R_ zpx  And, ZeroPageX   */
            0x36 => run!(rol, rw_zero_page_x),  /*bytes: 2 cycles: 6  ____P=>____P RW zpx  Rol, ZeroPageX   */
            0x37 => run!(rla, rw_zero_page_x),  /*bytes: 2 cycles: 6  A___P=>A___P RW zpx  Rla, ZeroPageX   */
            0x38 => run!(sec, implied),         /*bytes: 1 cycles: 2  _____=>____P __      Sec, Implied     */
            0x39 => run!(and, r_absolute_y),    /*bytes: 3 cycles: 4* A____=>A___P R_ absy And, AbsoluteY   */
            0x3A => run!(mop, implied),         /*bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied     */
            0x3B => run!(rla, rw_absolute_y),   /*bytes: 3 cycles: 7  A___P=>A___P RW absy Rla, AbsoluteY   */
            0x3C => run!(dop, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX   */
            0x3D => run!(and, r_absolute_x),    /*bytes: 3 cycles: 4* A____=>A___P R_ absx And, AbsoluteX   */
            0x3E => run!(rol, rw_absolute_x),   /*bytes: 3 cycles: 7  ____P=>____P RW absx Rol, AbsoluteX   */
            0x3F => run!(rla, rw_absolute_x),   /*bytes: 3 cycles: 7  A___P=>A___P RW absx Rla, AbsoluteX   */
            0x40 => run!(rti),                  /*bytes: X cycles: 6  ___S_=>___SP __      Rti, Implied     */
            0x41 => run!(eor, r_indirect_x),    /*bytes: 2 cycles: 6  A____=>____P R_ izx  Eor, IndirectX   */
            0x42 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x43 => run!(sre, rw_indirect_x),   /*bytes: 2 cycles: 8  A____=>____P RW izx  Sre, IndirectX   */
            0x44 => run!(dop, r_zero_page),     /*bytes: 2 cycles: 3  _____=>_____ R_ zp   Nop, ZeroPage    */
            0x45 => run!(eor, r_zero_page),     /*bytes: 2 cycles: 3  A____=>A___P R_ zp   Eor, ZeroPage    */
            0x46 => run!(lsr, rw_zero_page),    /*bytes: 2 cycles: 5  _____=>____P RW zp   Lsr, ZeroPage    */
            0x47 => run!(sre, rw_zero_page),    /*bytes: 2 cycles: 5  A____=>A___P RW zp   Sre, ZeroPage    */
            0x48 => run!(pha, w_stack),         /*bytes: 1 cycles: 3  A__S_=>___S_ _W      Pha, Implied     */
            0x49 => run!(eor, immediate),       /*bytes: 2 cycles: 2  A____=>A___P __      Eor, Immediate   */
            0x4A => run!(lsr_acc, accumulator), /*bytes: 1 cycles: 2  A____=>A___P __      Lsr, Implied     */
            0x4B => run!(alr, immediate),       /*bytes: 2 cycles: 2  A____=>A___P __      Alr, Immediate   */
            0x4C => run!(jmp_absolute),         /*bytes: X cycles: 3  _____=>_____ __      Jmp, Absolute    */
            0x4D => run!(eor, r_absolute),      /*bytes: 3 cycles: 4  A____=>A___P R_ abs  Eor, Absolute    */
            0x4E => run!(lsr, rw_absolute),     /*bytes: 3 cycles: 6  _____=>____P RW abs  Lsr, Absolute    */
            0x4F => run!(sre, rw_absolute),     /*bytes: 3 cycles: 6  A____=>A___P RW abs  Sre, Absolute    */
            0x50 => run!(bvc),                  /*bytes: 2 cycles: 2* ____P=>_____ __      Bvc, Relative    */
            0x51 => run!(eor, r_indirect_y),    /*bytes: 2 cycles: 5* A____=>____P R_ izy  Eor, IndirectY   */
            0x52 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x53 => run!(sre, rw_indirect_y),   /*bytes: 2 cycles: 8  A____=>____P RW izy  Sre, IndirectY   */
            0x54 => run!(dop, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX   */
            0x55 => run!(eor, r_zero_page_x),   /*bytes: 2 cycles: 4  A____=>A___P R_ zpx  Eor, ZeroPageX   */
            0x56 => run!(lsr, rw_zero_page_x),  /*bytes: 2 cycles: 6  _____=>____P RW zpx  Lsr, ZeroPageX   */
            0x57 => run!(sre, rw_zero_page_x),  /*bytes: 2 cycles: 6  A____=>A___P RW zpx  Sre, ZeroPageX   */
            0x58 => run!(cli, implied),         /*bytes: 1 cycles: 2  _____=>____P __      Cli, Implied     */
            0x59 => run!(eor, r_absolute_y),    /*bytes: 3 cycles: 4* A____=>A___P R_ absy Eor, AbsoluteY   */
            0x5A => run!(mop, implied),         /*bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied     */
            0x5B => run!(sre, rw_absolute_y),   /*bytes: 3 cycles: 7  A____=>A___P RW absy Sre, AbsoluteY   */
            0x5C => run!(dop, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX   */
            0x5D => run!(eor, r_absolute_x),    /*bytes: 3 cycles: 4* A____=>A___P R_ absx Eor, AbsoluteX   */
            0x5E => run!(lsr, rw_absolute_x),   /*bytes: 3 cycles: 7  _____=>____P RW absx Lsr, AbsoluteX   */
            0x5F => run!(sre, rw_absolute_x),   /*bytes: 3 cycles: 7  A____=>A___P RW absx Sre, AbsoluteX   */
            0x60 => run!(rts),                  /*bytes: X cycles: 6  ___S_=>___S_ __      Rts, Implied     */
            0x61 => run!(adc, r_indirect_x),    /*bytes: 2 cycles: 6  A___P=>A___P R_ izx  Adc, IndirectX   */
            0x62 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x63 => run!(rra, rw_indirect_x),   /*bytes: 2 cycles: 8  A___P=>A___P RW izx  Rra, IndirectX   */
            0x64 => run!(dop, r_zero_page),     /*bytes: 2 cycles: 3  _____=>_____ R_ zp   Nop, ZeroPage    */
            0x65 => run!(adc, r_zero_page),     /*bytes: 2 cycles: 3  A___P=>A___P R_ zp   Adc, ZeroPage    */
            0x66 => run!(ror, rw_zero_page),    /*bytes: 2 cycles: 5  ____P=>____P RW zp   Ror, ZeroPage    */
            0x67 => run!(rra, rw_zero_page),    /*bytes: 2 cycles: 5  A___P=>A___P RW zp   Rra, ZeroPage    */
            0x68 => run!(pla, r_stack),         /*bytes: 1 cycles: 4  ___S_=>A__SP __      Pla, Implied     */
            0x69 => run!(adc, immediate),       /*bytes: 2 cycles: 2  A___P=>A___P __      Adc, Immediate   */
            0x6A => run!(ror_acc, accumulator), /*bytes: 1 cycles: 2  A___P=>A___P __      Ror, Accumulator */
            0x6B => run!(arr, immediate),       /*bytes: 2 cycles: 2  A___P=>A___P __      Arr, Immediate   */
            0x6C => run!(jmp_indirect),         /*bytes: X cycles: 5  _____=>_____ __      Jmp, Indirect    */
            0x6D => run!(adc, r_absolute),      /*bytes: 3 cycles: 4  A___P=>A___P R_ abs  Adc, Absolute    */
            0x6E => run!(ror, rw_absolute),     /*bytes: 3 cycles: 6  ____P=>____P RW abs  Ror, Absolute    */
            0x6F => run!(rra, rw_absolute),     /*bytes: 3 cycles: 6  A___P=>A___P RW abs  Rra, Absolute    */
            0x70 => run!(bvs),                  /*bytes: 2 cycles: 2* _____=>_____ __      Bvs, Relative    */
            0x71 => run!(adc, r_indirect_y),    /*bytes: 2 cycles: 5* A___P=>A___P R_ izy  Adc, IndirectY   */
            0x72 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x73 => run!(rra, rw_indirect_y),   /*bytes: 2 cycles: 8  A___P=>A___P RW izy  Rra, IndirectY   */
            0x74 => run!(dop, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX   */
            0x75 => run!(adc, r_zero_page_x),   /*bytes: 2 cycles: 4  A___P=>A___P R_ zpx  Adc, ZeroPageX   */
            0x76 => run!(ror, rw_zero_page_x),  /*bytes: 2 cycles: 6  ____P=>____P RW zpx  Ror, ZeroPageX   */
            0x77 => run!(rra, rw_zero_page_x),  /*bytes: 2 cycles: 6  A___P=>A___P RW zpx  Rra, ZeroPageX   */
            0x78 => run!(sei, implied),         /*bytes: 1 cycles: 2  _____=>____P __      Sei, Implied     */
            0x79 => run!(adc, r_absolute_y),    /*bytes: 3 cycles: 4* A___P=>A___P R_ absy Adc, AbsoluteY   */
            0x7A => run!(mop, implied),         /*bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied     */
            0x7B => run!(rra, rw_absolute_y),   /*bytes: 3 cycles: 7  A___P=>A___P RW absy Rra, AbsoluteY   */
            0x7C => run!(dop, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX   */
            0x7D => run!(adc, r_absolute_x),    /*bytes: 3 cycles: 4* A___P=>A___P R_ absx Adc, AbsoluteX   */
            0x7E => run!(ror, rw_absolute_x),   /*bytes: 3 cycles: 7  ____P=>____P RW absx Ror, AbsoluteX   */
            0x7F => run!(rra, rw_absolute_x),   /*bytes: 3 cycles: 7  A___P=>A___P RW absx Rra, AbsoluteX   */
            0x80 => run!(dop, immediate),       /*bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate   */
            0x81 => run!(sta, w_indirect_x),    /*bytes: 2 cycles: 6  A____=>_____ RW izx  Sta, IndirectX   */
            0x82 => run!(dop, immediate),       /*bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate   */
            0x83 => run!(sax, w_indirect_x),    /*bytes: 2 cycles: 6  _____=>_____ RW izx  Sax, IndirectX   */
            0x84 => run!(sty, w_zero_page),     /*bytes: 2 cycles: 3  __Y__=>_____ _W zp   Sty, ZeroPage    */
            0x85 => run!(sta, w_zero_page),     /*bytes: 2 cycles: 3  A____=>_____ _W zp   Sta, ZeroPage    */
            0x86 => run!(stx, w_zero_page),     /*bytes: 2 cycles: 3  _X___=>_____ _W zp   Stx, ZeroPage    */
            0x87 => run!(sax, w_zero_page),     /*bytes: 2 cycles: 3  _____=>_____ _W zp   Sax, ZeroPage    */
            0x88 => run!(dey, implied),         /*bytes: 1 cycles: 2  __Y__=>__Y_P __      Dey, Implied     */
            0x89 => run!(dop, immediate),       /*bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate   */
            0x8A => run!(txa, implied),         /*bytes: 1 cycles: 2  _X___=>A___P __      Txa, Implied     */
            0x8B => run!(xaa, immediate),       /*bytes: 2 cycles: 2  _____=>A___P __      Xaa, Immediate   */
            0x8C => run!(sty, w_absolute),      /*bytes: 3 cycles: 4  __Y__=>_____ _W abs  Sty, Absolute    */
            0x8D => run!(sta, w_absolute),      /*bytes: 3 cycles: 4  A____=>_____ _W abs  Sta, Absolute    */
            0x8E => run!(stx, w_absolute),      /*bytes: 3 cycles: 4  _X___=>_____ _W abs  Stx, Absolute    */
            0x8F => run!(sax, w_absolute),      /*bytes: 3 cycles: 4  _____=>_____ _W abs  Sax, Absolute    */
            0x90 => run!(bcc),                  /*bytes: 2 cycles: 2* ____P=>_____ __      Bcc, Relative    */
            0x91 => run!(sta, w_indirect_y),    /*bytes: 2 cycles: 6  A____=>_____ RW izy  Sta, IndirectY   */
            0x92 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0x93 => run!(ahx, rw_indirect_y),   /*bytes: 2 cycles: 6  _____=>_____ RW izy  Ahx, IndirectY   */
            0x94 => run!(sty, w_zero_page_x),   /*bytes: 2 cycles: 4  __Y__=>_____ RW zpx  Sty, ZeroPageX   */
            0x95 => run!(sta, w_zero_page_x),   /*bytes: 2 cycles: 4  A____=>_____ RW zpx  Sta, ZeroPageX   */
            0x96 => run!(stx, w_zero_page_y),   /*bytes: 2 cycles: 4  _X___=>_____ RW zpy  Stx, ZeroPageY   */
            0x97 => run!(sax, w_zero_page_y),   /*bytes: 2 cycles: 4  _____=>_____ RW zpy  Sax, ZeroPageY   */
            0x98 => run!(tya, implied),         /*bytes: 1 cycles: 2  __Y__=>A___P __      Tya, Implied     */
            0x99 => run!(sta, w_absolute_y),    /*bytes: 3 cycles: 5  A____=>_____ RW absy Sta, AbsoluteY   */
            0x9A => run!(txs, implied),         /*bytes: X cycles: 2  _X___=>___S_ __      Txs, Implied     */
            0x9B => run!(tas, w_absolute_y),    /*bytes: X cycles: 5  __Y__=>___S_ _W      Tas, AbsoluteY   */
            0x9C => run!(shy, w_absolute_x),    /*bytes: 3 cycles: 5  __Y__=>_____ _W absx Shy, AbsoluteX   */
            0x9D => run!(sta, w_absolute_x),    /*bytes: 3 cycles: 5  A____=>_____ RW absx Sta, AbsoluteX   */
            0x9E => run!(shx, w_absolute_y),    /*bytes: 3 cycles: 5  _X___=>_____ _W absy Shx, AbsoluteY   */
            0x9F => run!(ahx, rw_absolute_x),   /*bytes: 3 cycles: 5  _____=>_____ RW absy Ahx, AbsoluteY   */
            0xA0 => run!(ldy, immediate),       /*bytes: 2 cycles: 2  _____=>__Y_P __      Ldy, Immediate   */
            0xA1 => run!(lda, r_indirect_x),    /*bytes: 2 cycles: 6  _____=>A___P R_ izx  Lda, IndirectX   */
            0xA2 => run!(ldx, immediate),       /*bytes: 2 cycles: 2  _____=>_X__P __      Ldx, Immediate   */
            0xA3 => run!(lax, r_indirect_x),    /*bytes: 2 cycles: 6  _____=>AX__P R_ izx  Lax, IndirectX   */
            0xA4 => run!(ldy, r_zero_page),     /*bytes: 2 cycles: 3  _____=>__Y_P R_ zp   Ldy, ZeroPage    */
            0xA5 => run!(lda, r_zero_page),     /*bytes: 2 cycles: 3  _____=>A___P R_ zp   Lda, ZeroPage    */
            0xA6 => run!(ldx, r_zero_page),     /*bytes: 2 cycles: 3  _____=>_X__P R_ zp   Ldx, ZeroPage    */
            0xA7 => run!(lax, r_zero_page),     /*bytes: 2 cycles: 3  _____=>AX__P R_ zp   Lax, ZeroPage    */
            0xA8 => run!(tay, implied),         /*bytes: 1 cycles: 2  A____=>__Y_P __      Tay, Implied     */
            0xA9 => run!(lda, immediate),       /*bytes: 2 cycles: 2  _____=>A___P __      Lda, Immediate   */
            0xAA => run!(tax, implied),         /*bytes: 1 cycles: 2  A____=>_X__P __      Tax, Implied     */
            0xAB => run!(lax, immediate),       /*bytes: 2 cycles: 2  A____=>AX__P __      Lax, Immediate   */
            0xAC => run!(ldy, r_absolute),      /*bytes: 3 cycles: 4  _____=>__Y_P R_ abs  Ldy, Absolute    */
            0xAD => run!(lda, r_absolute),      /*bytes: 3 cycles: 4  _____=>A___P R_ abs  Lda, Absolute    */
            0xAE => run!(ldx, r_absolute),      /*bytes: 3 cycles: 4  _____=>_X__P R_ abs  Ldx, Absolute    */
            0xAF => run!(lax, r_absolute),      /*bytes: 3 cycles: 4  _____=>AX__P R_ abs  Lax, Absolute    */
            0xB0 => run!(bcs),                  /*bytes: 2 cycles: 2* _____=>_____ __      Bcs, Relative    */
            0xB1 => run!(lda, r_indirect_y),    /*bytes: 2 cycles: 5* _____=>A___P R_ izy  Lda, IndirectY   */
            0xB2 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0xB3 => run!(lax, r_indirect_y),    /*bytes: 2 cycles: 5* _____=>AX__P R_ izy  Lax, IndirectY   */
            0xB4 => run!(ldy, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>__Y_P R_ zpx  Ldy, ZeroPageX   */
            0xB5 => run!(lda, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>A___P R_ zpx  Lda, ZeroPageX   */
            0xB6 => run!(ldx, r_zero_page_y),   /*bytes: 2 cycles: 4  _____=>_X__P R_ zpy  Ldx, ZeroPageY   */
            0xB7 => run!(lax, r_zero_page_y),   /*bytes: 2 cycles: 4  _____=>AX__P R_ zpy  Lax, ZeroPageY   */
            0xB8 => run!(clv, implied),         /*bytes: 1 cycles: 2  _____=>____P __      Clv, Implied     */
            0xB9 => run!(lda, r_absolute_y),    /*bytes: 3 cycles: 4* _____=>A___P R_ absy Lda, AbsoluteY   */
            0xBA => run!(tsx, implied),         /*bytes: 1 cycles: 2  ___S_=>_X__P __      Tsx, Implied     */
            0xBB => run!(las, r_absolute_y),    /*bytes: 3 cycles: 4* ___S_=>AX_SP R_ absy Las, AbsoluteY   */
            0xBC => run!(ldy, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>__Y_P R_ absx Ldy, AbsoluteX   */
            0xBD => run!(lda, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>A___P R_ absx Lda, AbsoluteX   */
            0xBE => run!(ldx, r_absolute_y),    /*bytes: 3 cycles: 4* _____=>_X__P R_ absy Ldx, AbsoluteY   */
            0xBF => run!(lax, r_absolute_y),    /*bytes: 3 cycles: 4* _____=>AX__P R_ absy Lax, AbsoluteY   */
            0xC0 => run!(cpy, immediate),       /*bytes: 2 cycles: 2  __Y__=>____P __      Cpy, Immediate   */
            0xC1 => run!(cmp, r_indirect_x),    /*bytes: 2 cycles: 6  A____=>____P R_ izx  Cmp, IndirectX   */
            0xC2 => run!(dop, immediate),       /*bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate   */
            0xC3 => run!(dcp, rw_indirect_x),   /*bytes: 2 cycles: 8  A____=>____P RW izx  Dcp, IndirectX   */
            0xC4 => run!(cpy, r_zero_page),     /*bytes: 2 cycles: 3  __Y__=>____P R_ zp   Cpy, ZeroPage    */
            0xC5 => run!(cmp, r_zero_page),     /*bytes: 2 cycles: 3  A____=>____P R_ zp   Cmp, ZeroPage    */
            0xC6 => run!(dec, rw_zero_page),    /*bytes: 2 cycles: 5  _____=>____P RW zp   Dec, ZeroPage    */
            0xC7 => run!(dcp, rw_zero_page),    /*bytes: 2 cycles: 5  A____=>____P RW zp   Dcp, ZeroPage    */
            0xC8 => run!(iny, implied),         /*bytes: 1 cycles: 2  __Y__=>__Y_P __      Iny, Implied     */
            0xC9 => run!(cmp, immediate),       /*bytes: 2 cycles: 2  A____=>____P __      Cmp, Immediate   */
            0xCA => run!(dex, implied),         /*bytes: 1 cycles: 2  _X___=>_X__P __      Dex, Implied     */
            0xCB => run!(axs, immediate),       /*bytes: 2 cycles: 2  _____=>_X__P __      Axs, Immediate   */
            0xCC => run!(cpy, r_absolute),      /*bytes: 3 cycles: 4  __Y__=>____P R_ abs  Cpy, Absolute    */
            0xCD => run!(cmp, r_absolute),      /*bytes: 3 cycles: 4  A____=>____P R_ abs  Cmp, Absolute    */
            0xCE => run!(dec, rw_absolute),     /*bytes: 3 cycles: 6  _____=>____P RW abs  Dec, Absolute    */
            0xCF => run!(dcp, rw_absolute),     /*bytes: 3 cycles: 6  A____=>____P RW abs  Dcp, Absolute    */
            0xD0 => run!(bne),                  /*bytes: 2 cycles: 2* ____P=>_____ __      Bne, Relative    */
            0xD1 => run!(cmp, r_indirect_y),    /*bytes: 2 cycles: 5* A____=>____P R_ izy  Cmp, IndirectY   */
            0xD2 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0xD3 => run!(dcp, rw_indirect_y),   /*bytes: 2 cycles: 8  A____=>____P RW izy  Dcp, IndirectY   */
            0xD4 => run!(dop, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX   */
            0xD5 => run!(cmp, r_zero_page_x),   /*bytes: 2 cycles: 4  A____=>____P R_ zpx  Cmp, ZeroPageX   */
            0xD6 => run!(dec, rw_zero_page_x),  /*bytes: 2 cycles: 6  _____=>____P RW zpx  Dec, ZeroPageX   */
            0xD7 => run!(dcp, rw_zero_page_x),  /*bytes: 2 cycles: 6  A____=>____P RW zpx  Dcp, ZeroPageX   */
            0xD8 => run!(cld, implied),         /*bytes: 1 cycles: 2  _____=>____P __      Cld, Implied     */
            0xD9 => run!(cmp, r_absolute_y),    /*bytes: 3 cycles: 4* A____=>____P R_ absy Cmp, AbsoluteY   */
            0xDA => run!(mop, implied),         /*bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied     */
            0xDB => run!(dcp, rw_absolute_y),   /*bytes: 3 cycles: 7  A____=>____P RW absy Dcp, AbsoluteY   */
            0xDC => run!(dop, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX   */
            0xDD => run!(cmp, r_absolute_x),    /*bytes: 3 cycles: 4* A____=>____P R_ absx Cmp, AbsoluteX   */
            0xDE => run!(dec, rw_absolute_x),   /*bytes: 3 cycles: 7  _____=>____P RW absx Dec, AbsoluteX   */
            0xDF => run!(dcp, rw_absolute_x),   /*bytes: 3 cycles: 7  A____=>____P RW absx Dcp, AbsoluteX   */
            0xE0 => run!(cpx, immediate),       /*bytes: 2 cycles: 2  _X___=>____P __      Cpx, Immediate   */
            0xE1 => run!(sbc, r_indirect_x),    /*bytes: 2 cycles: 6  A___P=>A___P R_ izx  Sbc, IndirectX   */
            0xE2 => run!(dop, immediate),       /*bytes: 2 cycles: 2  _____=>_____ __      Nop, Immediate   */
            0xE3 => run!(isc, rw_indirect_x),   /*bytes: 2 cycles: 8  A___P=>A___P RW izx  Isc, IndirectX   */
            0xE4 => run!(cpx, r_zero_page),     /*bytes: 2 cycles: 3  _X___=>____P R_ zp   Cpx, ZeroPage    */
            0xE5 => run!(sbc, r_zero_page),     /*bytes: 2 cycles: 3  A___P=>A___P R_ zp   Sbc, ZeroPage    */
            0xE6 => run!(inc, rw_zero_page),    /*bytes: 2 cycles: 5  _____=>____P RW zp   Inc, ZeroPage    */
            0xE7 => run!(isc, rw_zero_page),    /*bytes: 2 cycles: 5  A___P=>A___P RW zp   Isc, ZeroPage    */
            0xE8 => run!(inx, implied),         /*bytes: 1 cycles: 2  _X___=>_X__P __      Inx, Implied     */
            0xE9 => run!(sbc, immediate),       /*bytes: 2 cycles: 2  A___P=>A___P __      Sbc, Immediate   */
            0xEA => run!(nop, implied),         /*bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied     */
            0xEB => run!(sbn, immediate),       /*bytes: 2 cycles: 2  A___P=>A___P __      Sbc, Immediate   */
            0xEC => run!(cpx, r_absolute),      /*bytes: 3 cycles: 4  _X___=>____P R_ abs  Cpx, Absolute    */
            0xED => run!(sbc, r_absolute),      /*bytes: 3 cycles: 4  A___P=>A___P R_ abs  Sbc, Absolute    */
            0xEE => run!(inc, rw_absolute),     /*bytes: 3 cycles: 6  _____=>____P RW abs  Inc, Absolute    */
            0xEF => run!(isc, rw_absolute),     /*bytes: 3 cycles: 6  A___P=>A___P RW abs  Isc, Absolute    */
            0xF0 => run!(beq),                  /*bytes: 2 cycles: 2* _____=>_____ __      Beq, Relative    */
            0xF1 => run!(sbc, r_indirect_y),    /*bytes: 2 cycles: 5* A___P=>A___P R_ izy  Sbc, IndirectY   */
            0xF2 => run!(kil),                  /*Crashes. Stops the cycle from advancing  Kil, Implied     */
            0xF3 => run!(isc, rw_indirect_y),   /*bytes: 2 cycles: 8  A___P=>A___P RW izy  Isc, IndirectY   */
            0xF4 => run!(dop, r_zero_page_x),   /*bytes: 2 cycles: 4  _____=>_____ R_ zpx  Nop, ZeroPageX   */
            0xF5 => run!(sbc, r_zero_page_x),   /*bytes: 2 cycles: 4  A___P=>A___P R_ zpx  Sbc, ZeroPageX   */
            0xF6 => run!(inc, rw_zero_page_x),  /*bytes: 2 cycles: 6  _____=>____P RW zpx  Inc, ZeroPageX   */
            0xF7 => run!(isc, rw_zero_page_x),  /*bytes: 2 cycles: 6  A___P=>A___P RW zpx  Isc, ZeroPageX   */
            0xF8 => run!(sed, implied),         /*bytes: 1 cycles: 2  _____=>____P __      Sed, Implied     */
            0xF9 => run!(sbc, r_absolute_y),    /*bytes: 3 cycles: 4* A___P=>A___P R_ absy Sbc, AbsoluteY   */
            0xFA => run!(mop, implied),         /*bytes: 1 cycles: 2  _____=>_____ __      Nop, Implied     */
            0xFB => run!(isc, rw_absolute_y),   /*bytes: 3 cycles: 7  A___P=>A___P RW absy Isc, AbsoluteY   */
            0xFC => run!(dop, r_absolute_x),    /*bytes: 3 cycles: 4* _____=>_____ R_ absx Nop, AbsoluteX   */
            0xFD => run!(sbc, r_absolute_x),    /*bytes: 3 cycles: 4* A___P=>A___P R_ absx Sbc, AbsoluteX   */
            0xFE => run!(inc, rw_absolute_x),   /*bytes: 3 cycles: 7  _____=>____P RW absx Inc, AbsoluteX   */
            0xFF => run!(isc, rw_absolute_x),   /*bytes: 3 cycles: 7  A___P=>A___P RW absx Isc, AbsoluteX   */
            _ => unimplemented!()
        }
    }

    pub fn step_instruction(&mut self) {
        loop {
            self.step();
            if self.reg.get_cycle() == cycle::LAST
                || self.reg.get_current_instr() == 0x22 // KIL
                { break; }
        }
    }

    // Step until a condition is met
    pub fn step_until(&mut self, condition: fn(&Self) -> bool) {
        loop {
            self.step();
            if condition(&self) { break; }
        }
    }

    pub fn reset(&mut self) {
        self.bus.borrow_mut().reset = true;
        while self.bus.borrow().reset { self.step(); }
    }

    // Run the instruction passed.
    // This has horrible side effects and should be used only for testing.
    pub fn run(&mut self, code: &[u8]) {
        let pc = self.reg.get_pc();

        // Copy into memory at PC
        for (i, &data) in code.iter().enumerate() {
            self.bus.borrow_mut().write_cpu(pc + (i as u16), data)
        }

        self.step_instruction()
    }

    // Run a full cycle of the OAM DMA
    pub fn run_oam_dma(&mut self) { self.step_instruction() }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "Cpu | clock: {:>6}, [ {:?} ]", self.clock, self.reg)
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter, "{:?}\n{}", self, self.bus.borrow())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use super::*;

    fn run(bus: Vec<u8>, size: i16, clock: u32, setup: fn(&mut Cpu), result: fn(&mut Cpu)) {
        let bus = Bus::with_mem(&bus);
        let bus_ref = Rc::new(RefCell::<Bus>::new(bus));
        let mut cpu = Cpu::new(bus_ref.clone());
        setup(&mut cpu);

        let mut check = Cpu {
            log: Default::default(),
            reg: cpu.reg.clone(),
            clock: clock + 7, // Account for reset routine
            bus: bus_ref.clone(),
            oam_transferring: false,
            resetting: false,
            interrupting: false,
        };

        // Force PC to zero
        cpu.reg.s_pc(0x00);
        check.reg.s_pc(cpu.reg.get_pc().wrapping_add(size as u16));

        // Run
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

        result(&mut check);

        assert_eq!(check.clock, cpu.clock, "\n\n{}", cpu);
        assert_eq!(check.reg, cpu.reg, "\n\n{}", cpu);
    }

    fn as_is(_: &mut Cpu) {}

    #[test]
    #[should_panic(expected = "Kil opcode finished running. Aborting program.")]
    fn kil() { run(vec![0x02], 0, 0, as_is, as_is); }

    #[test]
    fn brk() {
        run(vec![0x00], 0, 7,
            as_is,
            |cpu| {
                cpu.reg.s_pc(0x00);
                cpu.reg.s_s(0xfa);
                cpu.bus.borrow_mut().write_cpu(0x01fd, 0x00);
                cpu.bus.borrow_mut().write_cpu(0x01fc, 0x02);
                cpu.bus.borrow_mut().write_cpu(0x01fb, 0x34);
            },
        );
    }

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

        #[test]
        fn indirect_y() {
            run(vec![0xB1, 0x03, 0xff, 0x05, 0x00, 0xff, 0xff, 0x10, 0xff, 0xff, 0xff], 2, 5,
                |cpu| { cpu.reg.s_y(0x02) },
                |cpu| { cpu.reg.s_a(0x10) },
            );
        }
    }
}
