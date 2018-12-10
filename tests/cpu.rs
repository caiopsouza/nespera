use nespera::console::Console;

#[test]
fn nestest() {
    let log = include_str!("resources/cpu/nestest.log");
    let file = include_bytes!("resources/cpu/nestest.nes")[..].to_owned();
    let console = Console::new(file);
    let mut cpu = console.cpu;
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

fn run_blargg(file: Vec<u8>, res: &'static str) {
    let console = Console::new(file);
    let mut cpu = console.cpu;

    while cpu.bus.borrow_mut().read(0x6000) == 0x00 { cpu.step(); }
    while cpu.bus.borrow_mut().read(0x6000) == 0x80 { cpu.step(); }

    let res = format!("{}\n\nPassed\n", res);
    assert_eq!(cpu.bus.borrow_mut().read_string(0x6005), res);
}

// TODO: 03-dummy_reads. Runs forever.
// TODO: 04-dummy_reads_apu.nes. APU not implemented.
#[cfg(test)]
mod instr_misc {
    use super::*;

    #[test]
    fn abs_x_wrap() {
        let file = include_bytes!("resources/cpu/instr_misc/01-abs_x_wrap.nes")[..].to_owned();
        run_blargg(file, "01-abs_x_wrap");
    }

    #[test]
    fn branch_wrap() {
        let file = include_bytes!("resources/cpu/instr_misc/02-branch_wrap.nes")[..].to_owned();
        run_blargg(file, "02-branch_wrap");
    }
}
