extern crate nespera;

mod cpu;

#[test]
fn clc() {
    run!(opc: [opc::Clc];
        reg: [p => flags::Flags::Carry.bits()];
        res: ["c" => false]);
}

#[test]
fn cld() {
    run!(opc: [opc::Cld];
        reg: [p => flags::Flags::DecimalMode.bits()];
        res: ["d" => false]);
}

#[test]
fn cli() {
    run!(opc: [opc::Cli];
        reg: [p => flags::Flags::InterruptDisable.bits()];
        res: ["i" => false]);
}

#[test]
fn clv() {
    run!(opc: [opc::Clv];
        reg: [p => flags::Flags::Overflow.bits()];
        res: ["v" => false]);
}

#[test]
fn sec() {
    run!(opc: [opc::Sec];
        reg: [p => flags::Flags::empty().bits()];
        res: ["c" => true]);
}

#[test]
fn sed() {
    run!(opc: [opc::Sed];
        reg: [p => flags::Flags::empty().bits()];
        res: ["d" => true]);
}

#[test]
fn sei() {
    run!(opc: [opc::Sei];
        reg: [p => flags::Flags::empty().bits()];
        res: ["i" => true]);
}
