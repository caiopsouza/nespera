use image::GenericImageView;

use nespera::console::Console;

#[test]
fn nestest() {
    let reference = image::open("tests/resources/ppu/nestest-reference.png").unwrap();

    let file = include_bytes!("resources/cpu/nestest.nes")[..].to_owned();
    let mut console = Console::new(file);

    console.run_frames(5, &mut Console::dismiss_log);

    let comparison = console.screen.pixels().zip(reference.pixels());
    for (pixel, (pixel_screen, (_, _, pixel_reference))) in comparison.enumerate() {
        assert_eq!(pixel_screen.data[..3], pixel_reference.data[..3], "at pixel {}", pixel);
    }
}
