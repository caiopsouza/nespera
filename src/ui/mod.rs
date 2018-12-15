use piston_window::*;

use crate::console::Console;

// Run the console on a window
pub fn run(console: &mut Console) {
    const SCALE: u32 = 3;
    const SCALE_AS_DOUBLE: f64 = SCALE as f64;

    let mut window: PistonWindow = WindowSettings::new("Nespera", [SCALE * 256, SCALE * 240])
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut settings = TextureSettings::new();
    settings.set_mag(Filter::Nearest);

    let mut canvas: G2dTexture = Texture::from_image(
        &mut window.factory,
        &console.screen,
        &settings,
    ).unwrap();

    while let Some(event) = window.next() {
        if let Some(_) = event.render_args() {
            console.run_frames(1);
            window.set_title(format!("Nespera | fps: {:.2}", console.fps));

            canvas.update(&mut window.encoder, &console.screen).unwrap();
            window.draw_2d(&event, |context, graphics| {
                image(&canvas,
                      context.transform.scale(SCALE_AS_DOUBLE, SCALE_AS_DOUBLE),
                      graphics);
            });
        }
    }
}
