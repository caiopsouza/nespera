extern crate nespera;

mod cpu;

#[test]
fn tax() {
    run!(opc: [opc::Tax];
        reg: [a => 244];
        res: ["x" => 244, "z" => false, "n" => true]);
}

#[test]
fn tay() {
    run!(opc: [opc::Tay];
        reg: [a => 244];
        res: ["y" => 244, "z" => false, "n" => true]);
}

#[test]
fn txa() {
    run!(opc: [opc::Txa];
        reg: [x => 244];
        res: ["a" => 244, "z" => false, "n" => true]);
}

#[test]
fn tya() {
    run!(opc: [opc::Tya];
        reg: [y => 244];
        res: ["a" => 244, "z" => false, "n" => true]);
}