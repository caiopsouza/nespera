extern crate nespera;

mod cpu;

use cpu::{msb, lsb};

#[test]
fn absolute() {
    run!(opc: [opc::Jmp::Absolute, 0x8a, 0x03];
        res: ["pc" => 0x038a]);
}

#[test]
fn indirect() {
    run!(opc: [opc::Jmp::Indirect, 0x8a, 0x03];
        ram: [0x038a => 0x98, 0x98 => 0x13, 0x99 => 0x01];
        res: ["pc" => 0x0113]);
}

#[test]
fn jsr() {
    run!(opc: [opc::Jsr, lsb(0x0125), msb(0x0125)];
        res: ["pc" => 0x125, "sp" => 0xfb, 0xfd => 0x02, 0xfc => 0x00]);
}


#[test]
fn rts() {
    run!(opc: [opc::Jsr, lsb(0x0125), msb(0x0125)];
        ram: [0x125 => opc::Rts];
        res: ["pc" => 0x03, "sp" => 0xfd, 0xfd => 0x02, 0xfc => 0x00]);
}
