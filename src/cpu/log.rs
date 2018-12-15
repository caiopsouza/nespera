use crate::cpu::reg::Reg;
use crate::utils::bits;

#[derive(Debug, Copy, Clone)]
pub enum AddrMode {
    Unknown,
    Implied,
    Accumulator,
    Immediate(u8),
    ZeroPage(u8, u8),
    ZeroPageX(u8, u8, u8),
    ZeroPageY(u8, u8, u8),
    Relative(u8, u16),
    Absolute(u16, u8),
    AbsoluteX(u16, u16, u8),
    AbsoluteY(u16, u16, u8),
    Direct(u16),
    Indirect(u16, u16),
    IndirectX(u8, u8, u16, u8),
    IndirectY(u8, u16, u16, u8),
}

fn f_u8(data: u8) -> String { format!("{:02X}", data) }

fn f_u16(data: u16) -> String { format!("{:04X}", data) }

impl AddrMode {
    fn operands(self) -> String {
        match self {
            AddrMode::Unknown => "?? ??".to_owned(),

            | AddrMode::Implied
            | AddrMode::Accumulator
            => "".to_owned(),

            | AddrMode::Immediate(op0)
            | AddrMode::ZeroPage(op0, _)
            | AddrMode::ZeroPageX(op0, _, _)
            | AddrMode::ZeroPageY(op0, _, _)
            | AddrMode::Relative(op0, _)
            | AddrMode::IndirectX(op0, _, _, _)
            | AddrMode::IndirectY(op0, _, _, _)
            => f_u8(op0),

            | AddrMode::Absolute(operand, _)
            | AddrMode::AbsoluteX(operand, _, _)
            | AddrMode::AbsoluteY(operand, _, _)
            | AddrMode::Direct(operand)
            | AddrMode::Indirect(operand, _)
            => format!("{} {}", f_u8(bits::low(operand)), f_u8(bits::high(operand))),
        }
    }

    fn operation(self) -> String {
        match self {
            AddrMode::Unknown => "???".to_owned(),
            AddrMode::Implied => "".to_owned(),
            AddrMode::Accumulator => "A".to_owned(),
            AddrMode::Immediate(data) => format!("#${}", f_u8(data)),
            AddrMode::ZeroPage(addr, data) => format!("${} = {}", f_u8(addr), f_u8(data)),
            AddrMode::ZeroPageX(operand, addr, data) => format!("${},X @ {} = {}", f_u8(operand), f_u8(addr), f_u8(data)),
            AddrMode::ZeroPageY(operand, addr, data) => format!("${},Y @ {} = {}", f_u8(operand), f_u8(addr), f_u8(data)),
            AddrMode::Absolute(addr, data) => format!("${} = {}", f_u16(addr), f_u8(data)),
            AddrMode::AbsoluteX(operand, addr, data) => format!("${},X @ {} = {}", f_u16(operand), f_u16(addr), f_u8(data)),
            AddrMode::AbsoluteY(operand, addr, data) => format!("${},Y @ {} = {}", f_u16(operand), f_u16(addr), f_u8(data)),
            AddrMode::Relative(_, addr) | AddrMode::Direct(addr) => format!("${}", f_u16(addr)),
            AddrMode::Indirect(operand, data) => format!("(${}) = {}", f_u16(operand), f_u16(data)),
            AddrMode::IndirectX(operand, index, addr, data) => format!("(${},X) @ {} = {} = {}", f_u8(operand), f_u8(index), f_u16(addr), f_u8(data)),
            AddrMode::IndirectY(operand, table, addr, data) => format!("(${}),Y = {} @ {} = {}", f_u8(operand), f_u16(table), f_u16(addr), f_u8(data)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Log {
    pub skip: bool,
    pub reg: Reg,
    pub unofficial: bool,
    pub mnemonic: &'static str,
    pub mode: AddrMode,
    pub dot: u32,
    pub scanline: i32,
}

impl Log {
    pub fn new() -> Self {
        Self {
            skip: true,
            reg: Reg::new(),
            unofficial: false,
            mnemonic: "???",
            mode: AddrMode::Unknown,
            dot: 0,
            scanline: 0,
        }
    }

    pub fn get(&self) -> String {
        if self.skip { return "".to_owned(); }

        format!("{}  {} {:5} {}{} {:<27} A:{} X:{} Y:{} P:{} SP:{} CYC:{:>3} SL:{}",
                f_u16(self.reg.get_pc()),
                f_u8(self.reg.get_current_instr()),
                self.mode.operands(),
                if self.unofficial { "*" } else { " " },
                self.mnemonic,
                self.mode.operation(),
                f_u8(self.reg.get_a()),
                f_u8(self.reg.get_x()),
                f_u8(self.reg.get_y()),
                f_u8(self.reg.get_p().into()),
                f_u8(self.reg.get_s()),
                self.dot,
                self.scanline
        )
    }
}

impl Default for Log { fn default() -> Self { Self::new() } }
