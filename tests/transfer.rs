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

#[test]
fn tsx() {
    run!(opc: [opc::Tsx];
        reg: [sp => 177];
        res: ["x" => 177, "z" => false, "n" => true]);
}

#[test]
fn txs() {
    run!(opc: [opc::Txs];
        reg: [x => 177];
        res: ["sp" => 177]);
}

#[test]
fn pha() {
    run!(opc: [opc::Pha];
        reg: [sp => 177, a => 157];
        res: ["sp" => 176, 177 => 157]);
}

#[test]
fn php() {
    run!(opc: [opc::Php];
        reg: [sp => 177, p => 157];
        res: ["sp" => 176, 177 => 157]);
}

#[test]
fn pla() {
    run!(opc: [opc::Pla];
        reg: [sp => 177];
        ram: [177 => 246];
        res: ["a" => 246, "sp" => 178, "z" => false, "n" => true]);
}

#[test]
fn plp() {
    run!(opc: [opc::Plp];
        reg: [sp => 177];
        ram: [177 => 246];
        res: [
            "p" => 246, "sp" => 178,
            "n" => true, "v" => true, "u" => true, "b" => true,
            "d" => false, "i" => true, "z" => true, "c" => false]);
}
