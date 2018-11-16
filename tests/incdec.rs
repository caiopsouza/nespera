extern crate nespera;

mod cpu;

use cpu::{lsb, msb};

mod inc {
    use super::*;

    #[test]
    fn zero_page() {
        run!(opc: [opc::Inc::ZeroPage, 0x0e];
            ram: [0x0e => 0xa7];
            res: [0x0e => 0xa8, "n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn zero_page_x() {
        run!(opc: [opc::Inc::ZeroPageX, 0x0e];
            reg: [x => 0x01];
            ram: [0x0f => 0xa7];
            res: [0x0f => 0xa8, "n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Inc::Absolute, lsb(0x143), msb(0x143)];
            ram: [0x143 => 0xa7];
            res: [0x143 => 0xa8, "n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute_x() {
        run!(opc: [opc::Inc::AbsoluteX, lsb(0x143), msb(0x143)];
            reg: [x => 0x01];
            ram: [0x144 => 0xa7];
            res: [0x144 => 0xa8, "n" => true, "z" => false, "c" => false]);
    }
}

mod dec {
    use super::*;

    #[test]
    fn zero_page() {
        run!(opc: [opc::Dec::ZeroPage, 0x0e];
            ram: [0x0e => 0xa7];
            res: [0x0e => 0xa6, "n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn zero_page_x() {
        run!(opc: [opc::Dec::ZeroPageX, 0x0e];
            reg: [x => 0x01];
            ram: [0x0f => 0xa7];
            res: [0x0f => 0xa6, "n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Dec::Absolute, lsb(0x143), msb(0x143)];
            ram: [0x143 => 0xa7];
            res: [0x143 => 0xa6, "n" => true, "z" => false, "c" => false]);
    }

    #[test]
    fn absolute_x() {
        run!(opc: [opc::Dec::AbsoluteX, lsb(0x143), msb(0x143)];
            reg: [x => 0x01];
            ram: [0x144 => 0xa7];
            res: [0x144 => 0xa6, "n" => true, "z" => false, "c" => false]);
    }
}

#[test]
fn inx() {
    run!(opc: [opc::Inx];
        reg: [x => 0xa7];
        res: ["x" => 0xa8, "n" => true, "z" => false, "c" => false]);
}

#[test]
fn dex() {
    run!(opc: [opc::Dex];
        reg: [x => 0xa7];
        res: ["x" => 0xa6, "n" => true, "z" => false, "c" => false]);
}

#[test]
fn iny() {
    run!(opc: [opc::Iny];
        reg: [y => 0xa7];
        res: ["y" => 0xa8, "n" => true, "z" => false, "c" => false]);
}

#[test]
fn dey() {
    run!(opc: [opc::Dey];
        reg: [y => 0xa7];
        res: ["y" => 0xa6, "n" => true, "z" => false, "c" => false]);
}
