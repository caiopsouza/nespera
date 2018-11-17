extern crate nespera;

mod cpu;

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
