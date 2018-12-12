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

fn run_blargg(file: Vec<u8>, expected: &'static str) {
    let mut console = Console::new(file);

    console.run_until_memory_is_not(0x6000, 0x00, &mut Console::dismiss_log);
    console.run_until_memory_is_not(0x6000, 0x80, &mut Console::dismiss_log);

    let expected = format!("{}\n\nPassed\n", expected);
    let res = console.cpu.bus.borrow_mut().read_string(0x6005);
    let debug = format!("{}", console.cpu);

    assert_eq!(res, expected, "\n{}\n{}", res, debug);
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

    #[test]
    fn dummy_reads() {
        let file = include_bytes!("resources/cpu/instr_misc/03-dummy_reads.nes")[..].to_owned();
        run_blargg(file, "03-dummy_reads");
    }

    #[test]
    fn dummy_reads_apu() {
        let file = include_bytes!("resources/cpu/instr_misc/04-dummy_reads_apu.nes")[..].to_owned();
        run_blargg(file, "03-dummy_reads_apu");
    }
}

#[cfg(test)]
mod instr_test {
    use super::*;

    #[test]
    fn basics() {
        let file = include_bytes!("resources/cpu/instr_test/01-basics.nes")[..].to_owned();
        run_blargg(file, "01-basics");
    }

    #[test]
    fn implied() {
        let file = include_bytes!("resources/cpu/instr_test/02-implied.nes")[..].to_owned();
        run_blargg(file, "02-implied");
    }

    #[test]
    fn immediate() {
        let file = include_bytes!("resources/cpu/instr_test/03-immediate.nes")[..].to_owned();
        run_blargg(file, "03-immediate");
    }

    #[test]
    fn zero_page() {
        let file = include_bytes!("resources/cpu/instr_test/04-zero_page.nes")[..].to_owned();
        run_blargg(file, "04-zero_page");
    }

    #[test]
    fn zp_xy() {
        let file = include_bytes!("resources/cpu/instr_test/05-zp_xy.nes")[..].to_owned();
        run_blargg(file, "05-zp_xy");
    }

    #[test]
    fn absolute() {
        let file = include_bytes!("resources/cpu/instr_test/06-absolute.nes")[..].to_owned();
        run_blargg(file, "06-absolute");
    }

    #[test]
    fn abs_xy() {
        let file = include_bytes!("resources/cpu/instr_test/07-abs_xy.nes")[..].to_owned();
        run_blargg(file, "07-abs_xy");
    }

    #[test]
    fn ind_x() {
        let file = include_bytes!("resources/cpu/instr_test/08-ind_x.nes")[..].to_owned();
        run_blargg(file, "08-ind_x");
    }

    #[test]
    fn ind_y() {
        let file = include_bytes!("resources/cpu/instr_test/09-ind_y.nes")[..].to_owned();
        run_blargg(file, "09-ind_y");
    }

    #[test]
    fn branches() {
        let file = include_bytes!("resources/cpu/instr_test/10-branches.nes")[..].to_owned();
        run_blargg(file, "10-branches");
    }

    #[test]
    fn stack() {
        let file = include_bytes!("resources/cpu/instr_test/11-stack.nes")[..].to_owned();
        run_blargg(file, "11-stack");
    }

    #[test]
    fn jmp_jsr() {
        let file = include_bytes!("resources/cpu/instr_test/12-jmp_jsr.nes")[..].to_owned();
        run_blargg(file, "12-jmp_jsr");
    }

    #[test]
    fn rts() {
        let file = include_bytes!("resources/cpu/instr_test/13-rts.nes")[..].to_owned();
        run_blargg(file, "13-rts");
    }

    #[test]
    fn rti() {
        let file = include_bytes!("resources/cpu/instr_test/14-rti.nes")[..].to_owned();
        run_blargg(file, "14-rti");
    }

    #[test]
    fn brk() {
        let file = include_bytes!("resources/cpu/instr_test/15-brk.nes")[..].to_owned();
        run_blargg(file, "15-brk");
    }

    #[test]
    fn special() {
        let file = include_bytes!("resources/cpu/instr_test/16-special.nes")[..].to_owned();
        run_blargg(file, "16-special");
    }
}