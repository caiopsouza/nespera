use nespera::console::Console;

#[test]
fn nestest() {
    let file = include_bytes!("resources/cpu/nestest.nes")[..].to_owned();
    let mut console = Console::new(file);
    console.cpu.reg.set_pc(0xc000);
    console.scanline = 241;
    console.run_log("tests/resources/cpu/nestest.log");
}

fn passed_message(test: &'static str) -> String { format!("\n{}\n\nPassed\n", test) }

fn run_blargg(file: Vec<u8>, expected: String) {
    let mut console = Console::new(file);

    console.run_until_cpu_memory_is_not(0x6000, 0x00);
    console.run_until_cpu_memory_is_not(0x6000, 0x80);

    let actual = console.cpu.bus.borrow_mut().read_cpu_string(0x6004);
    let debug = format!("{}", console.cpu);

    assert_eq!(expected, actual, "\n{}\n{}", actual, debug);
}

// TODO: 03-dummy_reads. Runs forever.
// TODO: 04-dummy_reads_apu.nes. Depends on the APU.
#[cfg(test)]
mod instr_misc {
    use super::*;

    #[test]
    fn abs_x_wrap() {
        let file = include_bytes!("resources/cpu/instr_misc/01-abs_x_wrap.nes")[..].to_owned();
        run_blargg(file, passed_message("01-abs_x_wrap"));
    }

    #[test]
    fn branch_wrap() {
        let file = include_bytes!("resources/cpu/instr_misc/02-branch_wrap.nes")[..].to_owned();
        run_blargg(file, passed_message("02-branch_wrap"));
    }
}

#[cfg(test)]
mod instr_test {
    use super::*;

    #[test]
    fn basics() {
        let file = include_bytes!("resources/cpu/instr_test/01-basics.nes")[..].to_owned();
        run_blargg(file, passed_message("01-basics"));
    }

    #[test]
    fn implied() {
        let file = include_bytes!("resources/cpu/instr_test/02-implied.nes")[..].to_owned();
        run_blargg(file, passed_message("02-implied"));
    }

    #[test]
    fn immediate() {
        let file = include_bytes!("resources/cpu/instr_test/03-immediate.nes")[..].to_owned();
        run_blargg(file, passed_message("03-immediate"));
    }

    #[test]
    fn zero_page() {
        let file = include_bytes!("resources/cpu/instr_test/04-zero_page.nes")[..].to_owned();
        run_blargg(file, passed_message("04-zero_page"));
    }

    #[test]
    fn zp_xy() {
        let file = include_bytes!("resources/cpu/instr_test/05-zp_xy.nes")[..].to_owned();
        run_blargg(file, passed_message("05-zp_xy"));
    }

    #[test]
    fn absolute() {
        let file = include_bytes!("resources/cpu/instr_test/06-absolute.nes")[..].to_owned();
        run_blargg(file, passed_message("06-absolute"));
    }

    #[test]
    fn abs_xy() {
        // This rom does not work properly.
        let file = include_bytes!("resources/cpu/instr_test/07-abs_xy.nes")[..].to_owned();
        run_blargg(file, "9C SYA abs,X\n9E SXA abs,Y\n\n07-abs_xy\n\nFailed\n".to_owned());
    }

    #[test]
    fn ind_x() {
        let file = include_bytes!("resources/cpu/instr_test/08-ind_x.nes")[..].to_owned();
        run_blargg(file, passed_message("08-ind_x"));
    }

    #[test]
    fn ind_y() {
        let file = include_bytes!("resources/cpu/instr_test/09-ind_y.nes")[..].to_owned();
        run_blargg(file, passed_message("09-ind_y"));
    }

    #[test]
    fn branches() {
        let file = include_bytes!("resources/cpu/instr_test/10-branches.nes")[..].to_owned();
        run_blargg(file, passed_message("10-branches"));
    }

    #[test]
    fn stack() {
        let file = include_bytes!("resources/cpu/instr_test/11-stack.nes")[..].to_owned();
        run_blargg(file, passed_message("11-stack"));
    }

    #[test]
    fn jmp_jsr() {
        let file = include_bytes!("resources/cpu/instr_test/12-jmp_jsr.nes")[..].to_owned();
        run_blargg(file, passed_message("12-jmp_jsr"));
    }

    #[test]
    fn rts() {
        let file = include_bytes!("resources/cpu/instr_test/13-rts.nes")[..].to_owned();
        run_blargg(file, passed_message("13-rts"));
    }

    #[test]
    fn rti() {
        let file = include_bytes!("resources/cpu/instr_test/14-rti.nes")[..].to_owned();
        run_blargg(file, passed_message("14-rti"));
    }

    #[test]
    fn brk() {
        let file = include_bytes!("resources/cpu/instr_test/15-brk.nes")[..].to_owned();
        run_blargg(file, passed_message("15-brk"));
    }

    #[test]
    fn special() {
        let file = include_bytes!("resources/cpu/instr_test/16-special.nes")[..].to_owned();
        run_blargg(file, passed_message("16-special"));
    }
}
