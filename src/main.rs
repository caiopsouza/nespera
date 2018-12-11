use env_logger::Builder;
use env_logger::Env;

use nespera::console::Console;

fn run_blargg(file: Vec<u8>, res: &'static str, log: &mut impl FnMut(&Console, String)) -> Console {
    let mut console = Console::new(file);

    console.run_until_memory_is_not(0x6000, 0x00, log);
    console.run_until_memory_is_not(0x6000, 0x80, log);

    let res = format!("{}\n\nPassed\n", res);
    assert_eq!(console.cpu.bus.borrow_mut().read_string(0x6005), res);

    console
}

fn main() {
    Builder::from_env(Env::default().default_filter_or("error")).init();

    let log = include_str!("../tests/resources/cpu/instr_test/11-stack.log");
    let mut log = log.split("\r\n").enumerate();
    let file = include_bytes!("../tests/resources/cpu/instr_test/11-stack.nes")[..].to_owned();
    let console = run_blargg(file,
                             "11-stack",
                             &mut |console, actual| {
                                 let (line, log) = log.next().unwrap();
                                 let expected = format!("{} {}", &log[0..8], &log[48..]);
                                 assert_eq!(actual, expected, "\nat line {}\n{}", line, console.cpu);
                             });

    println!("{}", console.cpu);
}
