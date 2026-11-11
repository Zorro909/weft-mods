//! Cogwork's pictures of oil, liquid tubes, tanks, the pumps and the pump's screen's liquid bar,
//! drawn here pixel by pixel so they can be changed and drawn again: `WRITE_ASSETS=1 cargo test
//! --release art` writes them (`generated`). Each family is a file of its own; the colors they
//! share are below.
//!
//! The pictures are small and few-colored like the rest of the mod's: a grid of letters with a
//! key for anything drawn by hand, and `noise` for a surface of one material.

mod gui;
mod liquids;
mod pump;
mod tank;
mod timber;

/// A color, red, green, blue and alpha.
pub type Rgba = [u8; 4];

pub const CLEAR: Rgba = [0, 0, 0, 0];
/// Iron, dark to light, as the item tubes have it.
pub const IRON: [Rgba; 4] = [
    [0x3a, 0x3a, 0x3e, 255],
    [0x4e, 0x4e, 0x52, 255],
    [0x7a, 0x7a, 0x7c, 255],
    [0xb0, 0xb0, 0xb0, 255],
];
/// Dark steel, dark to light, as the excavator's sides have it.
pub const STEEL: [Rgba; 5] = [
    [0x22, 0x24, 0x28, 255],
    [0x2e, 0x32, 0x38, 255],
    [0x38, 0x3c, 0x43, 255],
    [0x46, 0x4b, 0x53, 255],
    [0x60, 0x66, 0x6e, 255],
];
/// Brass, dark to light, as the torque shaft has it.
pub const BRASS: [Rgba; 5] = [
    [0x6e, 0x4a, 0x1a, 255],
    [0x8a, 0x62, 0x24, 255],
    [0xb0, 0x82, 0x32, 255],
    [0xd4, 0xa6, 0x48, 255],
    [0xec, 0xc8, 0x70, 255],
];
/// Copper, dark to light.
pub const COPPER: [Rgba; 5] = [
    [0x5a, 0x30, 0x1c, 255],
    [0x8f, 0x4f, 0x2e, 255],
    [0xa8, 0x60, 0x38, 255],
    [0xc4, 0x76, 0x48, 255],
    [0xf0, 0xb0, 0x80, 255],
];
/// Oil, dark to its sheen.
pub const OIL: [Rgba; 5] = [
    [0x0b, 0x09, 0x07, 255],
    [0x14, 0x10, 0x0c, 255],
    [0x1d, 0x17, 0x12, 255],
    [0x33, 0x2b, 0x3c, 255],
    [0x4a, 0x42, 0x30, 255],
];
/// Glass: its glints, and the faint metal in a liquid tube's.
pub const GLINT: Rgba = [0xe4, 0xec, 0xf0, 255];
pub const GLASS_METAL: Rgba = [0x9c, 0xa8, 0xb2, 255];

/// A picture: `width` by `height` pixels, row by row from the top.
#[derive(Clone)]
pub struct Picture {
    width: usize,
    height: usize,
    pixels: Vec<Rgba>,
}

impl Picture {
    /// A 16 by 16 picture all of `color`.
    pub fn filled(color: Rgba) -> Picture {
        Picture::sized(16, 16, color)
    }

    pub fn sized(width: usize, height: usize, color: Rgba) -> Picture {
        Picture {
            width,
            height,
            pixels: vec![color; width * height],
        }
    }

    /// A picture drawn as `rows` of letters, each a color of `key`; any other letter is clear.
    pub fn drawn(rows: &[&str], key: &[(char, Rgba)]) -> Picture {
        let mut picture = Picture::sized(rows[0].len(), rows.len(), CLEAR);
        for (y, row) in rows.iter().enumerate() {
            assert_eq!(
                row.len(),
                picture.width,
                "row {y} of a drawing is not as wide"
            );
            for (x, letter) in row.chars().enumerate() {
                if let Some(&(_, color)) = key.iter().find(|(at, _)| *at == letter) {
                    picture.set(x, y, color);
                }
            }
        }
        picture
    }

    /// A 16 by 16 surface of `colors`, each pixel one of them by `noise`, the first most often.
    pub fn surface(colors: &[Rgba], salt: u32) -> Picture {
        let mut picture = Picture::filled(CLEAR);
        picture.each(|x, y, _| {
            let roll = noise(x, y, salt) % 8;
            // Half the pixels the first color, the rest spread over the others.
            let pick = if roll < 4 {
                0
            } else {
                1 + roll as usize % (colors.len() - 1).max(1)
            };
            colors[pick.min(colors.len() - 1)]
        });
        picture
    }

    pub fn get(&self, x: usize, y: usize) -> Rgba {
        self.pixels[y * self.width + x]
    }

    pub fn set(&mut self, x: usize, y: usize, color: Rgba) {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x] = color;
        }
    }

    /// Sets every pixel to what `paint` makes of where it is and what it was.
    pub fn each(&mut self, paint: impl Fn(usize, usize, Rgba) -> Rgba) {
        for y in 0..self.height {
            for x in 0..self.width {
                let now = self.get(x, y);
                self.set(x, y, paint(x, y, now));
            }
        }
    }

    /// Fills the rectangle from (`x0`, `y0`) to before (`x1`, `y1`) with `color`.
    pub fn rect(&mut self, (x0, y0): (usize, usize), (x1, y1): (usize, usize), color: Rgba) {
        for y in y0..y1 {
            for x in x0..x1 {
                self.set(x, y, color);
            }
        }
    }

    /// Puts `other` on top, its top left corner at (`x0`, `y0`), its clear pixels leaving this.
    pub fn overlay(&mut self, other: &Picture, (x0, y0): (usize, usize)) {
        for y in 0..other.height {
            for x in 0..other.width {
                let color = other.get(x, y);
                if color[3] > 0 {
                    self.set(x0 + x, y0 + y, color);
                }
            }
        }
    }

    /// Pictures of the same width one under the other: the frames of an animation.
    pub fn stacked(frames: &[Picture]) -> Picture {
        let width = frames[0].width;
        let mut pixels = Vec::new();
        for frame in frames {
            assert_eq!(frame.width, width, "frames of one width");
            pixels.extend_from_slice(&frame.pixels);
        }
        let height = pixels.len() / width;
        Picture {
            width,
            height,
            pixels,
        }
    }

    /// The picture as a PNG file.
    pub fn png(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        let mut encoder = png::Encoder::new(&mut bytes, self.width as u32, self.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("a PNG header");
        let data: Vec<u8> = self.pixels.iter().flatten().copied().collect();
        writer.write_image_data(&data).expect("a PNG's pixels");
        drop(writer);
        bytes
    }
}

/// A number from where a pixel is and a `salt`, the same every time: what makes a surface
/// rough without a random generator.
pub fn noise(x: usize, y: usize, salt: u32) -> u32 {
    let mut n = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ salt;
    n ^= n >> 15;
    n = n.wrapping_mul(0x2C1B_3C6D);
    n ^= n >> 12;
    n
}

#[test]
fn the_pictures_are_what_is_drawn_here() {
    let all = liquids::all()
        .into_iter()
        .chain(pump::all())
        .chain(tank::all())
        .chain(timber::all())
        .chain(gui::all());
    for (path, picture) in all {
        let path = crate::generated::asset(&format!("textures/{path}.png"));
        crate::generated::save_or_check(&path, &picture.png());
    }
}

#[test]
fn a_drawing_keeps_its_letters_where_they_are() {
    let drawn = Picture::drawn(&["a.", ".a"], &[('a', IRON[0])]);
    assert_eq!(drawn.get(0, 0), IRON[0], "top left");
    assert_eq!(drawn.get(1, 0), CLEAR, "a letter without a color is clear");
    assert_eq!(drawn.get(1, 1), IRON[0], "bottom right");
}
