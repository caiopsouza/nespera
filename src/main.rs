use log::LevelFilter;

use nespera::cartridge::Cartridge;
use nespera::console::Console;
use nespera::cpu::log::setup;
use nespera::ui;
use nespera::ui::palette::Palette;

fn main() {
    setup(LevelFilter::Off, 0);
    let cartridge = Cartridge::from_file("tests/resources/roms/Balloon Fight (JU).nes").unwrap();
    let palette = Palette::from_file("tests/resources/palettes/RP2C03.pal").unwrap();
    let mut console = Console::new(cartridge);
    ui::run(&mut console, &palette);
}
