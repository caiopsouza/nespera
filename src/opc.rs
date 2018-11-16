#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

pub mod Lda {
    pub const Immediate: u8 = 0xa9;
    pub const ZeroPage: u8 = 0xa5;
    pub const ZeroPageX: u8 = 0xb5;
    pub const Absolute: u8 = 0xad;
    pub const AbsoluteX: u8 = 0xbd;
    pub const AbsoluteY: u8 = 0xb9;
    pub const IndirectX: u8 = 0xa1;
    pub const IndirectY: u8 = 0xb1;
}

pub mod Ldx {
    pub const Immediate: u8 = 0xa2;
    pub const ZeroPage: u8 = 0xa6;
    pub const ZeroPageY: u8 = 0xb6;
    pub const Absolute: u8 = 0xae;
    pub const AbsoluteY: u8 = 0xbe;
}

pub mod Ldy {
    pub const Immediate: u8 = 0xa0;
    pub const ZeroPage: u8 = 0xa4;
    pub const ZeroPageX: u8 = 0xb4;
    pub const Absolute: u8 = 0xac;
    pub const AbsoluteX: u8 = 0xbc;
}

pub mod Sta {
    pub const ZeroPage: u8 = 0x85;
    pub const ZeroPageX: u8 = 0x95;
    pub const Absolute: u8 = 0x8d;
    pub const AbsoluteX: u8 = 0x9d;
    pub const AbsoluteY: u8 = 0x99;
    pub const IndirectX: u8 = 0x81;
    pub const IndirectY: u8 = 0x91;
}

pub mod Stx {
    pub const ZeroPage: u8 = 0x86;
    pub const ZeroPageY: u8 = 0x96;
    pub const Absolute: u8 = 0x8e;
}

pub mod Sty {
    pub const ZeroPage: u8 = 0x84;
    pub const ZeroPageX: u8 = 0x94;
    pub const Absolute: u8 = 0x8c;
}

pub const Tax: u8 = 0xaa;
pub const Tay: u8 = 0xa8;
pub const Txa: u8 = 0x8a;
pub const Tya: u8 = 0x98;
pub const Tsx: u8 = 0xba;
pub const Txs: u8 = 0x9a;

pub const Pha: u8 = 0x48;
pub const Php: u8 = 0x08;
pub const Pla: u8 = 0x68;
pub const Plp: u8 = 0x28;

pub mod And {
    pub const Immediate: u8 = 0x29;
    pub const ZeroPage: u8 = 0x25;
    pub const ZeroPageX: u8 = 0x35;
    pub const Absolute: u8 = 0x2d;
    pub const AbsoluteX: u8 = 0x3d;
    pub const AbsoluteY: u8 = 0x39;
    pub const IndirectX: u8 = 0x21;
    pub const IndirectY: u8 = 0x31;
}

pub mod Ora {
    pub const Immediate: u8 = 0x09;
    pub const ZeroPage: u8 = 0x05;
    pub const ZeroPageX: u8 = 0x15;
    pub const Absolute: u8 = 0x0d;
    pub const AbsoluteX: u8 = 0x1d;
    pub const AbsoluteY: u8 = 0x19;
    pub const IndirectX: u8 = 0x01;
    pub const IndirectY: u8 = 0x11;
}

pub mod Eor {
    pub const Immediate: u8 = 0x49;
    pub const ZeroPage: u8 = 0x45;
    pub const ZeroPageX: u8 = 0x55;
    pub const Absolute: u8 = 0x4d;
    pub const AbsoluteX: u8 = 0x5d;
    pub const AbsoluteY: u8 = 0x59;
    pub const IndirectX: u8 = 0x41;
    pub const IndirectY: u8 = 0x51;
}

pub mod Bit {
    pub const ZeroPage: u8 = 0x24;
    pub const Absolute: u8 = 0x2c;
}

pub mod Adc {
    pub const Immediate: u8 = 0x69;
    pub const ZeroPage: u8 = 0x65;
    pub const ZeroPageX: u8 = 0x75;
    pub const Absolute: u8 = 0x6d;
    pub const AbsoluteX: u8 = 0x7d;
    pub const AbsoluteY: u8 = 0x79;
    pub const IndirectX: u8 = 0x61;
    pub const IndirectY: u8 = 0x71;
}

pub mod Sbc {
    pub const Immediate: u8 = 0xe9;
    pub const ZeroPage: u8 = 0xe5;
    pub const ZeroPageX: u8 = 0xf5;
    pub const Absolute: u8 = 0xed;
    pub const AbsoluteX: u8 = 0xfd;
    pub const AbsoluteY: u8 = 0xf9;
    pub const IndirectX: u8 = 0xe1;
    pub const IndirectY: u8 = 0xf1;
}