extern crate nespera;

use nespera::loader::ines::INes;
use nespera::hardware::nes::Nes;

fn main() {
    let log = include_str!("../tests/resources/nestest.log");
    let rom = Vec::<u8>::from(&include_bytes!("../tests/resources/nestest.nes")[..]);
    let mut nes: Nes = INes::new(rom).unwrap().into();

    for (line, text) in log.split("\r\n").enumerate() {
        let res = format!("a: {:02x}, x: {:02x}, y: {:02x}, p: {:02x}, sp: {:02x}",
                          nes.cpu.a, nes.cpu.x, nes.cpu.y, nes.cpu.p, nes.cpu.sp);
        assert_eq!(res, text);
        eprintln!("{:04} | {:02x} | {:?}", line, nes.mem.peek_at(nes.cpu.pc), nes.cpu);

        nes.step();
    }
}
