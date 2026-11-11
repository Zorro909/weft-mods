//! The mod's GUI pictures: the tiles a quest stands on, the mark under the shown one, and the
//! mod's icon. They are small enough to be written from here, a letter a pixel. After a change,
//! `WRITE_ALMANAC_PICTURES=1 cargo test --release pictures` writes them again; without it, the
//! test fails on a file that is not what is drawn here.

use std::path::{Path, PathBuf};

/// Set to anything, it has the test write the files instead of comparing them.
const WRITE: &str = "WRITE_ALMANAC_PICTURES";

/// Side of a tile in pixels, as base's advancement tiles.
const TILE: usize = 26;

/// Red, green, blue, opacity.
type Color = [u8; 4];
const CLEAR: Color = [0, 0, 0, 0];

/// A tile's rings from the outside in, then what fills it: a pixel each.
const TILES: [(&str, [u32; 4]); 4] = [
    ("almanac_tile", [0x434343, 0x9aa0b8, 0x6b6f86, 0x282828]),
    (
        "almanac_tile_locked",
        [0x434343, 0x343434, 0x282828, 0x282828],
    ),
    (
        "almanac_tile_ready",
        [0x434343, 0x7fdc6a, 0x4f9a45, 0x282828],
    ),
    (
        "almanac_tile_done",
        [0x434343, 0xf0cc5a, 0xc5923f, 0x282828],
    ),
];

/// The mod's icon, a book: `.` clear, `d` its dark edge, `b` and `l` its cover, `g` and `o` the
/// gold plate on it, `p` and `s` its pages, `r` its ribbon.
const ICON: [&str; 16] = [
    "................",
    "..dddddddddddd..",
    ".dbbbbbbbbbbbpd.",
    ".dbllllllllllpd.",
    ".dblgggggggllpd.",
    ".dblgooooogllpd.",
    ".dblgggggggllpd.",
    ".dbllllllllllpd.",
    ".dbllllllllllpd.",
    ".dblllgggllllpd.",
    ".dbllgoooglllpd.",
    ".dblllgggllllpd.",
    ".dbllllllllllpd.",
    ".dbbbbbrrbbbbsd.",
    "..dddddrrddddd..",
    ".......r.r......",
];

fn solid(color: u32) -> Color {
    let [_, red, green, blue] = color.to_be_bytes();
    [red, green, blue, 255]
}

fn icon_color(letter: u8) -> Color {
    match letter {
        b'd' => solid(0x3a2414),
        b'b' => solid(0x6b3f1f),
        b'l' => solid(0x8d5a2e),
        b'g' => solid(0xf0cc5a),
        b'o' => solid(0xc5923f),
        b'p' => solid(0xe8e0c8),
        b's' => solid(0xb8b098),
        b'r' => solid(0xc0392b),
        _ => CLEAR,
    }
}

/// A picture: its width, and its pixels row by row.
struct Picture {
    width: usize,
    pixels: Vec<Color>,
}

impl Picture {
    /// A square tile of `rings`, the last filling what the others leave.
    fn tile(rings: [u32; 4]) -> Picture {
        let pixels = (0..TILE * TILE)
            .map(|at| {
                let (x, y) = (at % TILE, at / TILE);
                let from_edge = x.min(y).min(TILE - 1 - x).min(TILE - 1 - y);
                solid(rings[from_edge.min(rings.len() - 1)])
            })
            .collect();
        Picture {
            width: TILE,
            pixels,
        }
    }

    /// The bar under the shown quest's tile, stretched to the tile's width where it is drawn.
    fn mark() -> Picture {
        Picture {
            width: 4,
            pixels: vec![solid(0xe8e8e8); 8],
        }
    }

    fn icon() -> Picture {
        let pixels = ICON
            .iter()
            .flat_map(|row| row.bytes().map(icon_color))
            .collect();
        Picture {
            width: ICON[0].len(),
            pixels,
        }
    }

    /// The picture as a PNG file's bytes.
    fn encoded(&self) -> Vec<u8> {
        let mut file = Vec::new();
        let height = self.pixels.len() / self.width;
        let mut encoder = png::Encoder::new(&mut file, self.width as u32, height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder
            .write_header()
            .expect("a PNG header goes into memory");
        writer
            .write_image_data(&self.pixels.concat())
            .expect("the pixels go into memory");
        drop(writer);
        file
    }
}

/// Every picture with the file it is kept in.
fn pictures() -> Vec<(PathBuf, Picture)> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/textures/gui");
    let tiles = TILES
        .iter()
        .map(|(name, rings)| (*name, Picture::tile(*rings)));
    tiles
        .chain([("almanac_mark", Picture::mark()), ("icon", Picture::icon())])
        .map(|(name, picture)| (folder.join(format!("{name}.png")), picture))
        .collect()
}

#[test]
fn the_pictures_are_what_is_drawn_here() {
    for (file, picture) in pictures() {
        let drawn = picture.encoded();
        if std::env::var_os(WRITE).is_some() {
            std::fs::write(&file, &drawn).expect("the picture's folder is there to write to");
        }
        let kept = std::fs::read(&file).unwrap_or_default();
        assert!(
            kept == drawn,
            "{} is not what `pictures.rs` draws; write it again with {WRITE}=1",
            file.display()
        );
    }
}

#[test]
fn the_icon_is_square() {
    assert!(
        ICON.iter().all(|row| row.len() == ICON.len()),
        "16 letters in each of 16 rows"
    );
}

#[test]
fn a_tile_is_framed_by_its_rings() {
    let tile = Picture::tile([1, 2, 3, 4]);
    let at = |x: usize, y: usize| tile.pixels[y * TILE + x];
    assert_eq!(at(0, 0), solid(1), "the corner");
    assert_eq!(at(1, 12), solid(2), "a pixel in");
    assert_eq!(at(12, TILE - 3), solid(3), "two in, from the far side");
    assert_eq!(at(12, 12), solid(4), "the middle");
}
