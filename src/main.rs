use std::io::Write;
use std::sync::RwLock;

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

use nespera::console::Console;

// Starts logging after the specified amount of logs has passed.
// Tracing is very verbose so you might need to limit how much is logged in order to speed up execution.
fn log_setup(level: LevelFilter, start_after: usize) {
    let counter = RwLock::new(0);
    Builder::new()
        .format(move |buf, record| {
            let mut counter = counter.write().unwrap();
            *counter += 1;
            if *counter < start_after { return Result::Ok(()); }
            writeln!(buf,
                     "{} [{}] - {}",
                     Local::now().format("%Y-%m-%dT%H:%M:%S"),
                     record.level(),
                     record.args()
            )
        })
        .filter(None, level)
        .init();
}

fn main() {
    log_setup(LevelFilter::Trace, 10_000_000);
    let file = include_bytes!("../tests/resources/cpu/instr_test/07-abs_xy.nes")[..].to_owned();
    let mut console = Console::new(file);
    console.run_log("tests/resources/cpu/instr_test/07-abs_xy.log");
    println!("{:?}", console);
}
