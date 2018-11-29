extern crate nespera;

use nespera::loader::ines::INes;
use nespera::mos6502::cpu::Cpu;

fn main() {
    let log = include_str!("../tests/resources/nestest.log");
    let rom = Vec::<u8>::from(&include_bytes!("../tests/resources/nestest.nes")[..]);
    let mut cpu: Cpu = INes::new(rom).unwrap().into();

    for (line, text) in log.split("\r\n").enumerate() {
        let p: u8 = cpu.reg.get_p().into();
        let res = format!("{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{: >3}",
                          cpu.reg.get_pc(), cpu.reg.get_a(), cpu.reg.get_x(),
                          cpu.reg.get_y(), p, cpu.reg.get_s(), (3 * cpu.get_clock()) % 341 /* PPU cycles*/);

        assert_eq!(res, text, "\n{:?}", cpu);

        eprintln!("{:04} | {:02x}, {:02x}, {:02x} | {:?}", line + 1,
                  cpu.reg.peek_addr(&mut cpu.addr_bus, cpu.reg.get_pc() as u16),
                  cpu.reg.peek_addr(&mut cpu.addr_bus, (cpu.reg.get_pc() + 1) as u16),
                  cpu.reg.peek_addr(&mut cpu.addr_bus, (cpu.reg.get_pc() + 2) as u16),
                  cpu.reg);

        cpu.step_instruction();
    }
}
