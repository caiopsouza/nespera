extern crate nespera;

mod cpu;

use cpu::{lsb, msb};

mod cmp {
    use super::*;

    #[test]
    fn immediate() {
        run!(opc: [opc::Cmp::Immediate, 0xfe];
            reg: [a => 0x80];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Cmp::ZeroPage, 0xa1];
            reg: [a => 0x80];
            ram: [0xa1 => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn zero_page_x() {
        run!(opc: [opc::Cmp::ZeroPageX, 0xa1];
            reg: [a => 0x80, x => 0x01];
            ram: [0xa2 => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Cmp::Absolute, lsb(0x2ee), msb(0x2ee)];
            reg: [a => 0x80];
            ram: [0x2ee => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute_x() {
        run!(opc: [opc::Cmp::AbsoluteX, lsb(0x2ee), msb(0x2ee)];
            reg: [a => 0x80, x => 0x01];
            ram: [0x2ef => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute_y() {
        run!(opc: [opc::Cmp::AbsoluteY, lsb(0x2ee), msb(0x2ee)];
            reg: [a => 0x80, y => 0x01];
            ram: [0x2ef => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn indirect_x() {
        run!(opc: [opc::Cmp::IndirectX, 0x2e];
            reg: [a => 0x80, x => 0x01];
            ram: [0x2f => 0x1e, 0x30 => 0x01, 0x011e => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn indirect_y() {
        run!(opc: [opc::Cmp::IndirectY, 0x2e];
            reg: [a => 0x80, y => 0x01];
            ram: [0x2e => 0x1e, 0x2f => 0x01, 0x011f => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }
}

mod cpx {
    use super::*;

    #[test]
    fn immediate() {
        run!(opc: [opc::Cpx::Immediate, 0xfe];
            reg: [x => 0x80];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Cpx::ZeroPage, 0xa1];
            reg: [x => 0x80];
            ram: [0xa1 => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Cpx::Absolute, lsb(0x2ee), msb(0x2ee)];
            reg: [x => 0x80];
            ram: [0x2ee => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }
}

mod cpy {
    use super::*;

    #[test]
    fn immediate() {
        run!(opc: [opc::Cpy::Immediate, 0xfe];
            reg: [y => 0x80];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Cpy::ZeroPage, 0xa1];
            reg: [y => 0x80];
            ram: [0xa1 => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Cpy::Absolute, lsb(0x2ee), msb(0x2ee)];
            reg: [y => 0x80];
            ram: [0x2ee => 0xfe];
            res: ["n" => true, "z" => false, "c" => false]);
    }
}
