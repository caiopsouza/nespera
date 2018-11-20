extern crate nespera;

mod cpu;

use std::num::Wrapping;
use cpu::{lsb, msb};

// Based on https://stackoverflow.com/a/8982549
#[cfg(test)]
mod flags {
    use super::*;

    #[cfg(test)]
    mod adc {
        use super::*;

        fn test(a: u8, param: u8, carry: bool, zero: bool, negative: bool, overflow: bool) {
            let res = Wrapping(a) + Wrapping(param);
            run!(opc: [opc::Adc::Immediate, param];
                reg: [a => a];
                res: ["a" => res.0, "c" => carry, "z" => zero, "n" => negative, "v" => overflow]);
        }

        #[test]
        fn empty() { test(0x7f, 0x00, false, false, false, false); }

        #[test]
        fn carry() { test(0xff, 0x7f, true, false, false, false); }

        #[test]
        fn zero() { test(0x00, 0x00, false, true, false, false); }

        #[test]
        fn zero_carry() { test(0xff, 0x01, true, true, false, false); }

        #[test]
        fn negative() { test(0xff, 0x00, false, false, true, false); }

        #[test]
        fn negative_carry() { test(0xff, 0xff, true, false, true, false); }

        #[test]
        fn carry_overflow() { test(0xff, 0x80, true, false, false, true); }

        #[test]
        fn zero_carry_overflow() { test(0x80, 0x80, true, true, false, true); }

        #[test]
        fn negative_overflow() { test(0x7f, 0x7f, false, false, true, true); }
    }

    // Carry is inverted for subtraction
    #[cfg(test)]
    mod sbc {
        use super::*;
        use std::num::Wrapping;

        fn test(a: u8, param: u8, carry: bool, zero: bool, negative: bool, overflow: bool) {
            let res = Wrapping(a) - Wrapping(param);
            run!(opc: [opc::Sec, opc::Sbc::Immediate, param];
                reg: [a => a];
                res: ["a" => res.0, "c" => !carry, "z" => zero, "n" => negative, "v" => overflow]);
        }

        #[test]
        fn empty() { test(0xff, 0xfe, false, false, false, false); }

        #[test]
        fn flags_c() { test(0x7e, 0xff, true, false, false, false); }

        #[test]
        fn flags_z() { test(0xff, 0xff, false, true, false, false); }

        #[test]
        fn flags_n() { test(0xff, 0x7f, false, false, true, false); }

        #[test]
        fn flags_nc() { test(0xfe, 0xff, true, false, true, false); }

        #[test]
        fn flags_v() { test(0xfe, 0x7f, false, false, false, true); }

        #[test]
        fn flags_ncv() { test(0x7f, 0xff, true, false, true, true); }
    }
}

mod adc {
    use super::*;

    #[test]
    fn immediate() {
        run!(opc: [opc::Sec, opc::Adc::Immediate, 0xfd];
            reg: [a => 0x7f];
            res: ["a" => 0x7d, "n" => false, "z" => false, "c" => true, "v" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Sec, opc::Adc::ZeroPage, 0x0e];
            reg: [a => 0x7f];
            ram: [0x0e => 0xff];
            res: ["a" => 0x7f, "n" => false, "z" => false, "c" => true, "v" => false]);
    }

    #[test]
    fn multiple_opcode() {
        run!(opc: [
                opc::Sec,
                opc::Adc::Immediate, 0x07,
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
            res: ["a" => 0x57, "n" => false, "z" => false, "c" => false, "v" => false]);
    }
}

mod sbc {
    use super::*;

    #[test]
    fn immediate() {
        run!(opc: [opc::Sbc::Immediate, 0x7f];
            reg: [a => 0xff];
            res: ["a" => 0x7f, "n" => false, "z" => false, "c" => true, "v" => true]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Sbc::ZeroPage, 0x0e];
            reg: [a => 0xff];
            ram: [0x0e => 0x7f];
            res: ["a" => 0x7f, "n" => false, "z" => false, "c" => true, "v" => true]);
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
            res: ["a" => 0xab, "n" => true, "z" => false, "c" => true, "v" => false]);
    }
}

mod asl {
    use super::*;

    #[test]
    fn accumulator() {
        run!(opc: [opc::Asl::Accumulator];
            reg: [a => 0b11011111];
            res: ["a" => 0b10111110, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Asl::ZeroPage, 0x2b];
            ram: [0x2b => 0b11011111];
            res: [0x2b => 0b10111110, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn zero_page_x() {
        run!(opc: [opc::Asl::ZeroPageX, 0x2b];
            reg: [x => 0x01];
            ram: [0x2c => 0b11011111];
            res: [0x2c => 0b10111110, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Asl::Absolute, lsb(0x2ee), msb(0x2ee)];
            ram: [0x2ee => 0b11011111];
            res: [0x2ee => 0b10111110, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn absolute_x() {
        run!(opc: [opc::Asl::AbsoluteX, lsb(0x2ee), msb(0x2ee)];
            reg: [x => 0x01];
            ram: [0x2ef => 0b11011111];
            res: [0x2ef => 0b10111110, "c" => true, "n" => true, "z" => false]);
    }
}

mod lsr {
    use super::*;

    #[test]
    fn accumulator() {
        run!(opc: [opc::Lsr::Accumulator];
            reg: [a => 0b11011111];
            res: ["a" => 0b01101111, "c" => true, "n" => false, "z" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Lsr::ZeroPage, 0x2b];
            ram: [0x2b => 0b11011111];
            res: [0x2b => 0b01101111, "c" => true, "n" => false, "z" => false]);
    }

    #[test]
    fn zero_page_x() {
        run!(opc: [opc::Lsr::ZeroPageX, 0x2b];
            reg: [x => 0x01];
            ram: [0x2c => 0b11011111];
            res: [0x2c => 0b01101111, "c" => true, "n" => false, "z" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Lsr::Absolute, lsb(0x2ee), msb(0x2ee)];
            ram: [0x2ee => 0b11011111];
            res: [0x2ee => 0b01101111, "c" => true, "n" => false, "z" => false]);
    }

    #[test]
    fn absolute_x() {
        run!(opc: [opc::Lsr::AbsoluteX, lsb(0x2ee), msb(0x2ee)];
            reg: [x => 0x01];
            ram: [0x2ef => 0b11011111];
            res: [0x2ef => 0b01101111, "c" => true, "n" => false, "z" => false]);
    }
}

mod rol {
    use super::*;

    #[test]
    fn accumulator() {
        run!(opc: [opc::Rol::Accumulator];
            reg: [p => flags::Flags::Carry.bits(), a => 0b11011111];
            res: ["a" => 0b10111111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Rol::ZeroPage, 0x2b];
            reg: [p => flags::Flags::Carry.bits()];
            ram: [0x2b => 0b11011111];
            res: [0x2b => 0b10111111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn zero_page_x() {
        run!(opc: [opc::Rol::ZeroPageX, 0x2b];
            reg: [p => flags::Flags::Carry.bits(), x => 0x01];
            ram: [0x2c => 0b11011111];
            res: [0x2c => 0b10111111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Rol::Absolute, lsb(0x2ee), msb(0x2ee)];
            reg: [p => flags::Flags::Carry.bits()];
            ram: [0x2ee => 0b11011111];
            res: [0x2ee => 0b10111111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn absolute_x() {
        run!(opc: [opc::Rol::AbsoluteX, lsb(0x2ee), msb(0x2ee)];
            reg: [p => flags::Flags::Carry.bits(), x => 0x01];
            ram: [0x2ef => 0b11011111];
            res: [0x2ef => 0b10111111, "c" => true, "n" => true, "z" => false]);
    }
}

mod ror {
    use super::*;

    #[test]
    fn accumulator() {
        run!(opc: [opc::Ror::Accumulator];
            reg: [p => flags::Flags::Carry.bits(), a => 0b11011111];
            res: ["a" => 0b11101111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn zero_page() {
        run!(opc: [opc::Ror::ZeroPage, 0x2b];
            reg: [p => flags::Flags::Carry.bits()];
            ram: [0x2b => 0b11011111];
            res: [0x2b => 0b11101111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn zero_page_x() {
        run!(opc: [opc::Ror::ZeroPageX, 0x2b];
            reg: [p => flags::Flags::Carry.bits(), x => 0x01];
            ram: [0x2c => 0b11011111];
            res: [0x2c => 0b11101111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn absolute() {
        run!(opc: [opc::Ror::Absolute, lsb(0x2ee), msb(0x2ee)];
            reg: [p => flags::Flags::Carry.bits()];
            ram: [0x2ee => 0b11011111];
            res: [0x2ee => 0b11101111, "c" => true, "n" => true, "z" => false]);
    }

    #[test]
    fn absolute_x() {
        run!(opc: [opc::Ror::AbsoluteX, lsb(0x2ee), msb(0x2ee)];
            reg: [p => flags::Flags::Carry.bits(), x => 0x01];
            ram: [0x2ef => 0b11011111];
            res: [0x2ef => 0b11101111, "c" => true, "n" => true, "z" => false]);
    }
}
