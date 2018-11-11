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
