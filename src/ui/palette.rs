use std::fs::File;
use std::io;
use std::io::Read;

const COLOR_AMOUNT: usize = 0x40;
const COLOR_DEPTH: usize = 3;

pub struct Palette {
    colors: [image::Rgba<u8>; COLOR_AMOUNT],
}

impl Palette {
    pub fn from_file(file: &str) -> Result<Self, io::Error> {
        let mut file = File::open(file)?;

        let mut rgb_colors = [0_u8; COLOR_AMOUNT * COLOR_DEPTH];
        file.read_exact(&mut rgb_colors)?;

        let mut colors = [image::Rgba::<u8>([0, 0, 0, 0]); COLOR_AMOUNT];
        for (rgb, color) in rgb_colors.chunks(COLOR_DEPTH).zip(colors.iter_mut()) {
            *color = image::Rgba::<u8>([rgb[0], rgb[1], rgb[2], 0xff])
        }

        Result::Ok(Self { colors })
    }

    // Map a list of pixels into an image
    pub fn map(&self, pixels: &[u8], image: &mut image::RgbaImage) {
        for (dest, &source) in image.pixels_mut().zip(pixels) {
            *dest = self.colors[source as usize];
        }
    }
}
