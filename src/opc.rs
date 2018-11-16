#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

// Load from A
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

// Load from X
pub mod Ldx {
    pub const Immediate: u8 = 0xa2;
    pub const ZeroPage: u8 = 0xa6;
    pub const ZeroPageY: u8 = 0xb6;
    pub const Absolute: u8 = 0xae;
    pub const AbsoluteY: u8 = 0xbe;
}

// Load from Y
pub mod Ldy {
    pub const Immediate: u8 = 0xa0;
    pub const ZeroPage: u8 = 0xa4;
    pub const ZeroPageX: u8 = 0xb4;
    pub const Absolute: u8 = 0xac;
    pub const AbsoluteX: u8 = 0xbc;
}

// Store in A
pub mod Sta {
    pub const ZeroPage: u8 = 0x85;
    pub const ZeroPageX: u8 = 0x95;
    pub const Absolute: u8 = 0x8d;
    pub const AbsoluteX: u8 = 0x9d;
    pub const AbsoluteY: u8 = 0x99;
    pub const IndirectX: u8 = 0x81;
    pub const IndirectY: u8 = 0x91;
}

// Store in X
pub mod Stx {
    pub const ZeroPage: u8 = 0x86;
    pub const ZeroPageY: u8 = 0x96;
    pub const Absolute: u8 = 0x8e;
}

// Store in X
pub mod Sty {
    pub const ZeroPage: u8 = 0x84;
    pub const ZeroPageX: u8 = 0x94;
    pub const Absolute: u8 = 0x8c;
}

// Transfer between registers
pub const Tax: u8 = 0xaa;
pub const Tay: u8 = 0xa8;
pub const Txa: u8 = 0x8a;
pub const Tya: u8 = 0x98;
pub const Tsx: u8 = 0xba;
pub const Txs: u8 = 0x9a;

// Stack operations
pub const Pha: u8 = 0x48;
pub const Php: u8 = 0x08;
pub const Pla: u8 = 0x68;
pub const Plp: u8 = 0x28;

// Bitwise And
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

// Bitwise Or
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

// Bitwise Xor
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

// Bit testing. Performs an "And" but doesn't change any value besides P
pub mod Bit {
    pub const ZeroPage: u8 = 0x24;
    pub const Absolute: u8 = 0x2c;
}

// Addition with carry
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

// Subtraction with borrow
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

// Increment memory
pub mod Inc {
    pub const ZeroPage: u8 = 0xe6;
    pub const ZeroPageX: u8 = 0xf6;
    pub const Absolute: u8 = 0xee;
    pub const AbsoluteX: u8 = 0xfe;
}

// Increment registers
pub const Inx: u8 = 0xe8;
pub const Iny: u8 = 0xc8;

// Decrement memory
pub mod Dec {
    pub const ZeroPage: u8 = 0xc6;
    pub const ZeroPageX: u8 = 0xd6;
    pub const Absolute: u8 = 0xce;
    pub const AbsoluteX: u8 = 0xde;
}

// Decrement registers
pub const Dex: u8 = 0xca;
pub const Dey: u8 = 0x88;

// Left shift
pub mod Asl {
    pub const Accumulator: u8 = 0x0a;
    pub const ZeroPage: u8 = 0x06;
    pub const ZeroPageX: u8 = 0x16;
    pub const Absolute: u8 = 0x0e;
    pub const AbsoluteX: u8 = 0x1e;
}

// Right shift
pub mod Lsr {
    pub const Accumulator: u8 = 0x4a;
    pub const ZeroPage: u8 = 0x46;
    pub const ZeroPageX: u8 = 0x56;
    pub const Absolute: u8 = 0x4e;
    pub const AbsoluteX: u8 = 0x5e;
}

// Left rotation
pub mod Rol {
    pub const Accumulator: u8 = 0x2a;
    pub const ZeroPage: u8 = 0x26;
    pub const ZeroPageX: u8 = 0x36;
    pub const Absolute: u8 = 0x2e;
    pub const AbsoluteX: u8 = 0x3e;
}

// Right rotation
pub mod Ror {
    pub const Accumulator: u8 = 0x6a;
    pub const ZeroPage: u8 = 0x66;
    pub const ZeroPageX: u8 = 0x76;
    pub const Absolute: u8 = 0x6e;
    pub const AbsoluteX: u8 = 0x7e;
}

// Compare values with A
pub mod Cmp {
    pub const Immediate: u8 = 0xc9;
    pub const ZeroPage: u8 = 0xc5;
    pub const ZeroPageX: u8 = 0xd5;
    pub const Absolute: u8 = 0xcd;
    pub const AbsoluteX: u8 = 0xdd;
    pub const AbsoluteY: u8 = 0xd9;
    pub const IndirectX: u8 = 0xc1;
    pub const IndirectY: u8 = 0xd1;
}

// Compare values with X
pub mod Cpx {
    pub const Immediate: u8 = 0xe0;
    pub const ZeroPage: u8 = 0xe4;
    pub const Absolute: u8 = 0xec;
}

// Compare values with Y
pub mod Cpy {
    pub const Immediate: u8 = 0xc0;
    pub const ZeroPage: u8 = 0xc4;
    pub const Absolute: u8 = 0xcc;
}

// Clear and set status flags
pub const Clc: u8 = 0x18;
pub const Cld: u8 = 0xd8;
pub const Cli: u8 = 0x58;
pub const Clv: u8 = 0xb8;
pub const Sec: u8 = 0x38;
pub const Sed: u8 = 0xf8;
pub const Sei: u8 = 0x78;

// Branches
pub const Bcs: u8 = 0xb0;
pub const Bcc: u8 = 0x90;
pub const Beq: u8 = 0xf0;
pub const Bne: u8 = 0xd0;
pub const Bmi: u8 = 0x30;
pub const Bpl: u8 = 0x10;
pub const Bvc: u8 = 0x50;
pub const Bvs: u8 = 0x70;
