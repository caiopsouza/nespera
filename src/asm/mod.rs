use crate::utils::bits;

pub enum Operand {
    Implied,
    Immediate(u8),
    ZeroPage(u8),
    ZeroPageX(u8),
    Absolute(u16),
    AbsoluteX(u16),
    AbsoluteY(u16),
    IndirectX(u8),
    IndirectY(u8),
}

fn byte(opcode: u8, data: u8) -> Vec<u8> { vec![opcode, data] }

fn word(opcode: u8, data: u16) -> Vec<u8> { vec![opcode, bits::low(data), bits::high(data)] }

pub fn kil() -> Vec<u8> { vec![0x02] }
pub fn brk() -> Vec<u8> { vec![0x00] }

// Load
pub fn lda(operand: Operand) -> Vec<u8> {
    match operand {
        Operand::Immediate(data) => { byte(0xA9, data) }
        Operand::ZeroPage(data) => { byte(0xA5, data) }
        Operand::ZeroPageX(data) => { byte(0xB5, data) }
        Operand::Absolute(data) => { word(0xAD, data) }
        Operand::AbsoluteX(data) => { word(0xBD, data) }
        Operand::AbsoluteY(data) => { word(0xB9, data) }
        Operand::IndirectX(data) => { byte(0xA1, data) }
        Operand::IndirectY(data) => { byte(0xB1, data) }
        _ => vec![]
    }
}

// Store
pub fn sta(operand: Operand) -> Vec<u8> {
    match operand {
        Operand::ZeroPage(data) => { byte(0x85, data) }
        Operand::ZeroPageX(data) => { byte(0x95, data) }
        Operand::Absolute(data) => { word(0x8D, data) }
        Operand::AbsoluteX(data) => { word(0x9D, data) }
        Operand::AbsoluteY(data) => { word(0x99, data) }
        Operand::IndirectX(data) => { byte(0x81, data) }
        Operand::IndirectY(data) => { byte(0x91, data) }
        _ => vec![]
    }
}
