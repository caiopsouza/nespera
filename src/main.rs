use env_logger::Builder;
use env_logger::Env;

use nespera::console::Console;
use nespera::console::RenderToDisk;

fn main() {
    Builder::from_env(Env::default().default_filter_or("error")).init();

    let file = include_bytes!("../tests/resources/cpu/nestest.nes")[..].to_owned();
    let mut console = Console::new(file);
    console.render_to_disk = RenderToDisk::Frame;
    console.run_frames(50, &mut Console::dismiss_log);
}
