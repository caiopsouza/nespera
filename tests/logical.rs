extern crate nespera;

mod cpu;

use cpu::{high, low};

mod and {
    use super::*;

    #[test]
    fn positive() {
        run!(opc: [opc::And::Immediate, 0b00111100];
            reg: [a => 0b00110011];
            res: ["a" => 0b00110000, "n" => false, "z" => false]);
    }

    #[test]
    fn zero() {
        run!(opc: [opc::And::Immediate, 0b10111100];
                reg: [a => 0b00000011];
                res: ["a" => 0b00000000, "n" => false, "z" => true]);
    }

    #[test]
    fn negative() {
        run!(opc: [opc::And::Immediate, 0b10111100];
            reg: [a => 0b10110011];
            res: ["a" => 0b10110000, "n" => true, "z" => false]);
    }

    #[test]
    fn multiple_opcode() {
        run!(opc: [
                opc::And::Immediate, 0b11111110,
                opc::And::ZeroPage, 0x098,
                opc::And::ZeroPageX, 0x0f1,
                opc::And::Absolute, high(0x2ee), low(0x2ee),
                opc::And::AbsoluteX, high(0x143), low(0x143),
                opc::And::AbsoluteY, high(0x2da), low(0x2da),
                opc::And::IndirectX, 0x0d7,
                opc::And::IndirectY, 0x0bb
            ];
            reg: [a => 0b11111111, x => 0x02, y => 0x01];
            ram: [
                0x098 => 0b00111100,
                0x0f3 => 0b11111000,
                0x2ee => 0b10110111,
                0x145 => 0b11111001,
                0x2db => 0b11011001,
                0x0d9 => 0x07, 0x0da => 0x10, 0x0710 => 0b10111101,
                0x0bb => 0x02, 0x0bc => 0x0e, 0x020f => 0b00011111
            ];
            res: ["a" => 0b00010000, "n" => false, "z" => false]);
    }
}

mod ora {
    use super::*;

    #[test]
    fn positive() {
        run!(opc: [opc::Ora::Immediate, 0b00111100];
            reg: [a => 0b00110011];
            res: ["a" => 0b00111111, "n" => false, "z" => false]);
    }

    #[test]
    fn zero() {
        run!(opc: [opc::Ora::Immediate, 0b00000000];
            reg: [a => 0b00000000];
            res: ["a" => 0b00000000, "n" => false, "z" => true]);
    }

    #[test]
    fn negative() {
        run!(opc: [opc::Ora::Immediate, 0b10111100];
            reg: [a => 0b10110011];
            res: ["a" => 0b10111111, "n" => true, "z" => false]);
    }

    #[test]
    fn multiple_opcode() {
        run!(opc: [
                opc::Ora::Immediate, 0b00000001,
                opc::Ora::ZeroPage, 0x098,
                opc::Ora::ZeroPageX, 0x0f1,
                opc::Ora::Absolute, high(0x2ee), low(0x2ee),
                opc::Ora::AbsoluteX, high(0x143), low(0x143),
                opc::Ora::AbsoluteY, high(0x2da), low(0x2da),
                opc::Ora::IndirectX, 0x0d7,
                opc::Ora::IndirectY, 0x0bb
            ];
            reg: [a => 0b00000000, x => 0x02, y => 0x01];
            ram: [
                0x098 => 0b11000011,
                0x0f3 => 0b00000111,
                0x2ee => 0b01001000,
                0x145 => 0b00000110,
                0x2db => 0b00100110,
                0x0d9 => 0x07, 0x0da => 0x10, 0x0710 => 0b01000010,
                0x0bb => 0x02, 0x0bc => 0x0e, 0x020f => 0b11100000
            ];
            res: ["a" => 0b11101111, "n" => true, "z" => false]);
    }
}

mod eor {
    use super::*;

    #[test]
    fn positive() {
        run!(opc: [opc::Eor::Immediate, 0b00111100];
            reg: [a => 0b00110011];
            res: ["a" => 0b00001111, "n" => false, "z" => false]);
    }

    #[test]
    fn zero() {
        run!(opc: [opc::Eor::Immediate, 0b11000000];
            reg: [a => 0b11000000];
            res: ["a" => 0b00000000, "n" => false, "z" => true]);
    }

    #[test]
    fn negative() {
        run!(opc: [opc::Eor::Immediate, 0b00111100];
                reg: [a => 0b10110011];
                res: ["a" => 0b10001111, "n" => true, "z" => false]);
    }

    #[test]
    fn multiple_opcode() {
        run!(opc: [
                opc::Eor::Immediate, 0b00000001,
                opc::Eor::ZeroPage, 0x098,
                opc::Eor::ZeroPageX, 0x0f1,
                opc::Eor::Absolute, high(0x2ee), low(0x2ee),
                opc::Eor::AbsoluteX, high(0x143), low(0x143),
                opc::Eor::AbsoluteY, high(0x2da), low(0x2da),
                opc::Eor::IndirectX, 0x0d7,
                opc::Eor::IndirectY, 0x0bb
            ];
            reg: [a => 0b00000000, x => 0x02, y => 0x01];
            ram: [
                0x098 => 0b11000011,
                0x0f3 => 0b00000111,
                0x2ee => 0b01001000,
                0x145 => 0b00000110,
                0x2db => 0b00100110,
                0x0d9 => 0x07, 0x0da => 0x10, 0x0710 => 0b01000010,
                0x0bb => 0x02, 0x0bc => 0x0e, 0x020f => 0b11100000
            ];
            res: ["a" => 0b00001111, "n" => false, "z" => false]);
    }
}

mod bit {
    use super::*;

    #[test]
    fn positive() {
        run!(opc: [opc::Bit::ZeroPage, 0x098];
            reg: [a => 0b00110011];
            ram: [0x098 => 0b00111100];
            res: ["n" => false, "z" => false, "o" => false]);
    }

    #[test]
    fn zero() {
        run!(opc: [opc::Bit::ZeroPage, 0x098];
            reg: [a => 0b00000011];
            ram: [0x098 => 0b10111100];
            res: ["n" => false, "z" => true, "o" => false]);
    }

    #[test]
    fn negative() {
        run!(opc: [opc::Bit::ZeroPage, 0x098];
            reg: [a => 0b10110011];
            ram: [0x098 => 0b10111100];
            res: ["n" => true, "z" => false, "o" => false]);
    }

    #[test]
    fn overflow() {
        run!(opc: [opc::Bit::ZeroPage, 0x098];
            reg: [a => 0b11110011];
            ram: [0x098 => 0b01111100];
            res: ["n" => false, "z" => false, "o" => true]);
    }

    #[test]
    fn negative_overflow() {
        run!(opc: [opc::Bit::ZeroPage, 0x098];
            reg: [a => 0b11110011];
            ram: [0x098 => 0b11111100];
            res: ["n" => true, "z" => false, "o" => true]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Bit::Absolute, high(0x398), low(0x398)];
            reg: [a => 0b11111111];
            ram: [0x398 => 0b10111100];
            res: ["n" => true, "z" => false, "o" => false]);
    }
}