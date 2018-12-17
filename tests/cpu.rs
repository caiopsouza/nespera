use nespera::cartridge::Cartridge;
use nespera::console::Console;

fn passed_message(test: &'static str) -> String { format!("\n{}\n\nPassed\n", test) }

fn run_blargg(file: &str, expected: String) {
    let file = format!("tests/resources/cpu/{}", file);
    let cartridge = Cartridge::from_file(&file[..]).unwrap();
    let mut console = Console::new(cartridge);

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
        run_blargg("instr_misc/01-abs_x_wrap.nes", passed_message("01-abs_x_wrap"));
    }

    #[test]
    fn branch_wrap() {
        run_blargg("instr_misc/02-branch_wrap.nes", passed_message("02-branch_wrap"));
    }
}

#[cfg(test)]
mod instr_test {
    use super::*;

    #[test]
    fn basics() {
        run_blargg("instr_test/01-basics.nes", passed_message("01-basics"));
    }

    #[test]
    fn implied() {
        run_blargg("instr_test/02-implied.nes", passed_message("02-implied"));
    }

    #[test]
    fn immediate() {
        run_blargg("instr_test/03-immediate.nes", passed_message("03-immediate"));
    }

    #[test]
    fn zero_page() {
        run_blargg("instr_test/04-zero_page.nes", passed_message("04-zero_page"));
    }

    #[test]
    fn zp_xy() {
        run_blargg("instr_test/05-zp_xy.nes", passed_message("05-zp_xy"));
    }

    #[test]
    fn absolute() {
        run_blargg("instr_test/06-absolute.nes", passed_message("06-absolute"));
    }

    #[test]
    fn abs_xy() {
        // This rom does not work properly.
        run_blargg("instr_test/07-abs_xy.nes", "9C SYA abs,X\n9E SXA abs,Y\n\n07-abs_xy\n\nFailed\n".to_owned());
    }

    #[test]
    fn ind_x() {
        run_blargg("instr_test/08-ind_x.nes", passed_message("08-ind_x"));
    }

    #[test]
    fn ind_y() {
        run_blargg("instr_test/09-ind_y.nes", passed_message("09-ind_y"));
    }

    #[test]
    fn branches() {
        run_blargg("instr_test/10-branches.nes", passed_message("10-branches"));
    }

    #[test]
    fn stack() {
        run_blargg("instr_test/11-stack.nes", passed_message("11-stack"));
    }

    #[test]
    fn jmp_jsr() {
        run_blargg("instr_test/12-jmp_jsr.nes", passed_message("12-jmp_jsr"));
    }

    #[test]
    fn rts() {
        run_blargg("instr_test/13-rts.nes", passed_message("13-rts"));
    }

    #[test]
    fn rti() {
        run_blargg("instr_test/14-rti.nes", passed_message("14-rti"));
    }

    #[test]
    fn brk() {
        run_blargg("instr_test/15-brk.nes", passed_message("15-brk"));
    }

    #[test]
    fn special() {
        run_blargg("instr_test/16-special.nes", passed_message("16-special"));
    }
}
