use env_logger::Builder;
use env_logger::Env;

use nespera::console::Console;

fn wait_until_running(console: &mut Console) {
    console.run_until_memory_is(0x6000, 0x80)
}

fn wait_until_ask_for_reset(console: &mut Console) {
    console.run_until_memory_is(0x6000, 0x81)
}

fn main() {
    Builder::from_env(Env::default().default_filter_or("debug")).init();

    let file = include_bytes!("../tests/resources/cpu/cpu_reset/registers.nes")[..].to_owned();
    let console = &mut Console::new(file);

    wait_until_running(console);
    wait_until_ask_for_reset(console);
    //console.run_until_memory_is(0x1fff, 0xb4);
    console.run_frames(5);
    console.cpu.reset();
    console.run_frames(5);
    println!("{}", console.cpu);
}
