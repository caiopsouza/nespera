use std::io::Write;
use std::sync::RwLock;

use chrono::Local;
use env_logger::Builder;
use log::LevelFilter;

use nespera::cartridge::Cartridge;
use nespera::console::Console;
use nespera::ui;
use nespera::ui::palette::Palette;

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
    log_setup(LevelFilter::Off, 0);
    let cartridge = Cartridge::from_file("tests/resources/roms/Balloon Fight (JU).nes").unwrap();
    let palette = Palette::from_file("tests/resources/palettes/RP2C03.pal").unwrap();
    let mut console = Console::new(cartridge);
    ui::run(&mut console, &palette);
}
