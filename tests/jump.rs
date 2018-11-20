extern crate nespera;

mod cpu;

use nespera::hardware::flags;
use nespera::hardware::opc;

use cpu::{msb, lsb};

#[test]
fn absolute() {
    run!(opc: [opc::Jmp::Absolute, lsb(0x038a), msb(0x038a)];
        res: ["pc" => 0x038a]);
}

#[test]
fn indirect() {
    run!(opc: [opc::Jmp::Indirect, lsb(0x038a), msb(0x038a)];
        ram: [0x038a => 0x98, 0x98 => 0x13, 0x99 => 0x01];
        res: ["pc" => 0x0113]);
}

#[test]
fn jsr() {
    run!(opc: [opc::Jsr, lsb(0x0125), msb(0x0125)];
        res: ["pc" => 0x125, "sp" => 0xfb, 0x1fc => 0x02, 0x1fb => 0x00]);
}

#[test]
fn rts() {
    run!(opc: [opc::Jsr, lsb(0x0125), msb(0x0125)];
        ram: [0x125 => opc::Rts];
        res: ["pc" => 0x03, "sp" => 0xfd, 0x1fc => 0x02, 0x1fb => 0x00]);
}

fn test_jump(opcode: u8, flags: flags::Flags) {
    run!(opc: [opcode, 0x10];
        reg: [p => flags.bits()];
        res: ["pc" => 0x12]);
}

fn test_dont_jump(opcode: u8, flags: flags::Flags) {
    run!(opc: [opcode, 0x10];
        reg: [p => flags.bits()];
        res: ["pc" => 0x02]);
}

#[test]
fn bcs_jump() { test_jump(opc::Bcs, flags::Flags::Carry); }

#[test]
fn bcs_dont_jump() { test_dont_jump(opc::Bcs, !flags::Flags::Carry); }

#[test]
fn bcc_jump() { test_jump(opc::Bcc, !flags::Flags::Carry); }

#[test]
fn bcc_dont_jump() { test_dont_jump(opc::Bcc, flags::Flags::Carry); }

#[test]
fn beq_jump() { test_jump(opc::Beq, flags::Flags::Zero); }

#[test]
fn beq_dont_jump() { test_dont_jump(opc::Beq, !flags::Flags::Zero); }

#[test]
fn bne_jump() { test_jump(opc::Bne, !flags::Flags::Zero); }

#[test]
fn bne_dont_jump() { test_dont_jump(opc::Bne, flags::Flags::Zero); }

#[test]
fn bmi_jump() { test_jump(opc::Bmi, flags::Flags::Negative); }

#[test]
fn bmi_dont_jump() { test_dont_jump(opc::Bmi, !flags::Flags::Negative); }

#[test]
fn bpl_jump() { test_jump(opc::Bpl, !flags::Flags::Negative); }

#[test]
fn bpl_dont_jump() { test_dont_jump(opc::Bpl, flags::Flags::Negative); }

#[test]
fn bvc_jump() { test_jump(opc::Bvc, !flags::Flags::Overflow); }

#[test]
fn bvc_dont_jump() { test_dont_jump(opc::Bvc, flags::Flags::Overflow); }

#[test]
fn bvs_jump() { test_jump(opc::Bvs, flags::Flags::Overflow); }

#[test]
fn bvs_dont_jump() { test_dont_jump(opc::Bvs, !flags::Flags::Overflow); }
