use std::cell::RefCell;
use std::rc::Rc;

use nespera::bus::Bus;
use nespera::cpu::Cpu;
use nespera::mapper::Cartridge;

#[test]
fn cpu() {
    let log = include_str!("resources/cpu/nestest.log");
    let file = include_bytes!("resources/cpu/nestest.nes")[..].to_owned();
    let cartridge = Cartridge::new(file).unwrap();
    let bus = Rc::new(RefCell::new(Bus::with_cartridge(cartridge)));
    let mut cpu = Cpu::new(bus);
    cpu.set_clock(0);

    // Starting point where the ROM won't access the PPU.
    cpu.reg.set_pc(0xc000);

    for (line, text) in log.split("\r\n").enumerate() {
        let ppu_cycle = (3 * cpu.get_clock()) % 341;

        let p: u8 = cpu.reg.get_p().into();
        let res = format!("{:04X} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{: >3}",
                          cpu.reg.get_pc(), cpu.reg.get_a(), cpu.reg.get_x(),
                          cpu.reg.get_y(), p, cpu.reg.get_s(), ppu_cycle);

        assert_eq!(res, text, "\nOn line: {}.\n{}", line + 1, cpu);

        cpu.step_instruction();
    }

    // Return from subroutine
    assert_eq!(0x60, cpu.reg.get_current_instr(), "\n\n{}", cpu);
}
