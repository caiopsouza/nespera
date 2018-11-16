extern crate nespera;

mod cpu;

use cpu::{lsb, msb};

mod adc {
    use super::*;

    #[test]
    fn immediate() {
        run!(opc: [opc::Adc::Immediate, 0xff];
            reg: [a => 0x80];
            res: ["a" => 0x7f, "n" => false, "z" => false, "c" => true, "o" => true]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Adc::ZeroPage, 0x0e];
            reg: [a => 0x80];
            ram: [0x0e => 0xff];
            res: ["a" => 0x7f, "n" => false, "z" => false, "c" => true, "o" => true]);
    }

    #[test]
    fn multiple_opcode() {
        run!(opc: [
                opc::Adc::Immediate, 0x08,
                opc::Adc::ZeroPage, 0x78,
                opc::Adc::ZeroPage, 0x5a,
                opc::Adc::ZeroPageX, 0x31,
                opc::Adc::Absolute, lsb(0x2ee), msb(0x2ee),
                opc::Adc::AbsoluteX, lsb(0x1ab), msb(0x1ab),
                opc::Adc::AbsoluteY, lsb(0x0aa), msb(0x0aa),
                opc::Adc::IndirectX, 0x98,
                opc::Adc::IndirectY, 0x74,
                opc::Adc::Immediate, 0x01
            ];
            reg: [a => 0x02, x => 0x01, y => 3];
            ram: [
                0x078 => 0x04,
                0x05a => 0x10,
                0x032 => 0x10,
                0x2ee => 0x05,
                0x1ac => 0x0f,
                0x0ad => 0x12,
                0x099 => 0x78, 0x09a => 0x01, 0x0178 => 0x03,
                0x074 => 0x47, 0x075 => 0x03, 0x034a => 0xfe
            ];
            res: ["a" => 0x57, "n" => false, "z" => false, "c" => false, "o" => false]);
    }
}

mod sbc {
    use super::*;

    #[test]
    fn immediate() {
        run!(opc: [opc::Sbc::Immediate, 0x7f];
            reg: [a => 0xff];
            res: ["a" => 0x7f, "n" => false, "z" => false, "c" => false, "o" => true]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Sbc::ZeroPage, 0x0e];
            reg: [a => 0xff];
            ram: [0x0e => 0x7f];
            res: ["a" => 0x7f, "n" => false, "z" => false, "c" => false, "o" => true]);
    }

    #[test]
    fn multiple_opcode() {
        run!(opc: [
                opc::Sbc::Immediate, 0x08,
                opc::Sbc::ZeroPage, 0x78,
                opc::Sbc::ZeroPage, 0x5a,
                opc::Sbc::ZeroPageX, 0x31,
                opc::Sbc::Absolute, lsb(0x2ee), msb(0x2ee),
                opc::Sbc::AbsoluteX, lsb(0x1ab), msb(0x1ab),
                opc::Sbc::AbsoluteY, lsb(0x0aa), msb(0x0aa),
                opc::Sbc::IndirectX, 0x98,
                opc::Sbc::IndirectY, 0x74,
                opc::Sbc::Immediate, 0x01
            ];
            reg: [a => 0x02, x => 0x01, y => 3];
            ram: [
                0x078 => 0x04,
                0x05a => 0x10,
                0x032 => 0x10,
                0x2ee => 0x05,
                0x1ac => 0x0f,
                0x0ad => 0x12,
                0x099 => 0x78, 0x09a => 0x01, 0x0178 => 0x03,
                0x074 => 0x47, 0x075 => 0x03, 0x034a => 0xfe
            ];
            res: ["a" => 0xa6, "n" => true, "z" => false, "c" => false, "o" => false]);
    }
}