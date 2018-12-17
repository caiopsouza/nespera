use piston_window::*;

use ::image::RgbaImage;

use crate::console::Console;
use crate::ppu;
use crate::ui::palette::Palette;

pub mod palette;

// Run the console on a window
pub fn run(console: &mut Console, palette: &Palette) {
    const SCALE: u32 = 3;
    const SCALE_AS_DOUBLE: f64 = SCALE as f64;

    let mut window: PistonWindow = WindowSettings::new("Nespera", [SCALE * ppu::SCREEN_WIDTH as u32, SCALE * ppu::SCREEN_HEIGHT as u32])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut settings = TextureSettings::new();
    settings.set_mag(Filter::Nearest);

    let mut screen = RgbaImage::new(ppu::SCREEN_WIDTH as u32, ppu::SCREEN_HEIGHT as u32);

    let mut canvas: G2dTexture = Texture::from_image(
        &mut window.factory,
        &screen,
        &settings,
    ).unwrap();

    while let Some(event) = window.next() {
        if event.render_args().is_some() {
            console.run_frames(1);

            window.set_title(format!("Nespera | fps: {:.2}", console.ppu.fps));

            palette.map(&console.ppu.screen, &mut screen);
            canvas.update(&mut window.encoder, &screen).unwrap();

            window.draw_2d(&event, |context, graphics| {
                image(&canvas,
                      context.transform.scale(SCALE_AS_DOUBLE, SCALE_AS_DOUBLE),
                      graphics);
            });
        }
    }
}
