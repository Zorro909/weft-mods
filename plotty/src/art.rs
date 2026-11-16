//! Plotty's pictures, drawn here as grids of letters so they can be changed and drawn again:
//! `WRITE_ASSETS=1 cargo test --release art` writes them; otherwise the tests fail on a file
//! that differs from what they would write, so a file changed by hand has to change here too.

use std::path::{Path, PathBuf};

/// A color, red, green, blue and alpha.
type Rgba = [u8; 4];

/// Set to anything, it has the tests write the files instead of comparing them.
const WRITE: &str = "WRITE_ASSETS";

/// Oak, dark to light.
const OAK: [Rgba; 3] = [
    [0x6b, 0x4a, 0x2a, 255],
    [0x8f, 0x67, 0x3c, 255],
    [0xb0, 0x84, 0x52, 255],
];
/// Stone, dark to light.
const STONE: [Rgba; 3] = [
    [0x55, 0x55, 0x58, 255],
    [0x7d, 0x7d, 0x80, 255],
    [0x9c, 0x9c, 0x9e, 255],
];
/// Gold, dark to light.
const GOLD: [Rgba; 3] = [
    [0x9a, 0x6a, 0x12, 255],
    [0xd6, 0xa2, 0x2c, 255],
    [0xf6, 0xd8, 0x6a, 255],
];
const PAPER: Rgba = [0xf2, 0xee, 0xe2, 255];
/// White cloth.
const CLOTH: Rgba = [0xee, 0xea, 0xe4, 255];
/// A till's dark wood, dark and light.
const TILL: [Rgba; 2] = [[0x4f, 0x2e, 0x1a, 255], [0x6e, 0x42, 0x26, 255]];
const INK: Rgba = [0x4a, 0x3a, 0x2a, 255];
const GRASS: [Rgba; 2] = [[0x4f, 0x7a, 0x2c, 255], [0x6a, 0x96, 0x3a, 255]];

/// Upright oak boards with a paper plaque framed in gold: a plot to claim.
const PLOT_POST: [&str; 16] = [
    "dmlmdmlmdmlmdmlm",
    "dmlmdmlmdmlmdmlm",
    "dmlmdmlmdmlmdmlm",
    "dmGGGGGGGGGGGGlm",
    "dmGppppppppppGlm",
    "dmGpiiiiiiiipGlm",
    "dmGppppppppppGlm",
    "dmGpiiiiippppGlm",
    "dmGppppppppppGlm",
    "dmGpiiiiiiippGlm",
    "dmGppppppppppGlm",
    "dmgGGGGGGGGGGglm",
    "dmlmdmlmdmlmdmlm",
    "dmlmdmlmdmlmdmlm",
    "dmlmdmlmdmlmdmlm",
    "dmlmdmlmdmlmdmlm",
];

/// Carved stone blocks with a gold diamond set in the middle: the heart of a city.
const CITY_STONE: [&str; 16] = [
    "kkkkkkkkkkkkkkkk",
    "ksssssskssssssSk",
    "ksSssssksssssssk",
    "kkkkkkkkkkkkkkkk",
    "ksssskkGkksssssk",
    "kssskGGyGGksssSk",
    "ksSkGGyyyGGksssk",
    "kkkGyyyYyyyGkkkk",
    "ksskGGyyyGGksssk",
    "ksssskGGyGksSssk",
    "kssssskkGkksssss",
    "kkkkkkkkkkkkkkkk",
    "ksssskssssssksSk",
    "kssSsksssssskssk",
    "ksssskssssssksss",
    "kkkkkkkkkkkkkkkk",
];

/// The mod's icon: a plot post on a plot of grass, four times as large.
const ICON: [&str; 16] = [
    "................",
    "................",
    "......GGGG......",
    ".....GppppG.....",
    ".....GpiipG.....",
    ".....GppppG.....",
    "......GGGG......",
    ".......ml.......",
    ".......ml.......",
    ".......ml.......",
    "..ssssssssssss..",
    "..sggggmlggggs..",
    "..sgGgggggggGs..",
    "..sggggggggggs..",
    "..ssssssssssss..",
    "................",
];

/// A Golden Woof: a gold coin with a paw print struck on it.
const WOOF: [&str; 16] = [
    "................",
    ".....yyyyyy.....",
    "...yyGGGGGGyy...",
    "..yGYGGGGGGGGy..",
    "..yYGGyGGyGGGy..",
    ".yGGGyyGGyyGGGy.",
    ".yGGGGGGGGGGGGy.",
    ".yGyyGGGGGGyyGy.",
    ".yGyyGGyyGGyyGy.",
    ".yGGGGyyyyGGGGy.",
    ".yGGGyyyyyyGGGy.",
    "..yGGyyyyyyGGy..",
    "..yGGGyyyyGGGy..",
    "...yyGGGGGGyy...",
    ".....yyyyyy.....",
    "................",
];

/// A stall's counter: planks laid across, light and dark by wood (`L`, `M`, `D`).
const STALL_PLANKS: [&str; 16] = [
    "MMMMMMMMMMMMMMMM",
    "MLLMMMMMMMMMLMMM",
    "MMMMMMMMMMMMMMMM",
    "MMMMMMLMMMMMMMMM",
    "DDDDDDDDDDDDDDDD",
    "MMMMMMMMMMMMMMMM",
    "MMMMLLMMMMMMMMMM",
    "MMMMMMMMMMMLLMMM",
    "DDDDDDDDDDDDDDDD",
    "MMMMMMMMMMMMMMMM",
    "MMMLMMMMMMMMMMMM",
    "MMMMMMMMMLLMMMMM",
    "DDDDDDDDDDDDDDDD",
    "MMMMMMMMMMMMMMMM",
    "MMMMMMLLMMMMMMLM",
    "DDDDDDDDDDDDDDDD",
];

/// A stall's stock crate: a frame braced with a cross.
const STALL_CRATE: [&str; 16] = [
    "DDDDDDDDDDDDDDDD",
    "DLLLLLLLLLLLLLLD",
    "DLDMMMMMMMMMMDLD",
    "DLMDMMMMMMMMDMLD",
    "DLMMDMMMMMMDMMLD",
    "DLMMMDMMMMDMMMLD",
    "DLMMMMDMMDMMMMLD",
    "DLMMMMMDDMMMMMLD",
    "DLMMMMMDDMMMMMLD",
    "DLMMMMDMMDMMMMLD",
    "DLMMMDMMMMDMMMLD",
    "DLMMDMMMMMMDMMLD",
    "DLMDMMMMMMMMDMLD",
    "DLDMMMMMMMMMMDLD",
    "DLLLLLLLLLLLLLLD",
    "DDDDDDDDDDDDDDDD",
];

/// A stall's post: grain running up.
const STALL_POST: [&str; 16] = [
    "DMMLMMDMMLMMDMML",
    "DMMLMMDMMLMMDMML",
    "DMMMMMDMMLMMDMMM",
    "DMMLMMDMMMMMDMML",
    "DMMLMMDMMLMMDMML",
    "DMMLMMMMMLMMDMML",
    "DMMLMMDMMLMMDMML",
    "DMMMMMDMMLMMMMML",
    "DMMLMMDMMLMMDMML",
    "DMMLMMDMMMMMDMML",
    "DMMLMMDMMLMMDMML",
    "MMMLMMDMMLMMDMML",
    "DMMLMMDMMLMMDMMM",
    "DMMLMMMMMLMMDMML",
    "DMMLMMDMMLMMDMML",
    "DMMLMMDMMLMMDMML",
];

/// A stall's awning: stripes of cloth in the wood's two colors (`a`, `b`).
const STALL_AWNING: [&str; 16] = [
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
    "aaaabbbbaaaabbbb",
];

/// The till on a register's counter: dark wood, gold keys and the drawer's handle.
const STALL_TILL: [&str; 16] = [
    "rrrrrrrrrrrrrrrr",
    "rRRRRRRRRRRRRRRr",
    "rRYYRYYRYYRYYRRr",
    "rRyyRyyRyyRyyRRr",
    "rRRRRRRRRRRRRRRr",
    "rRYYRYYRYYRYYRRr",
    "rRyyRyyRyyRyyRRr",
    "rRRRRRRRRRRRRRRr",
    "rrrrrrrrrrrrrrrr",
    "rRRRRRRRRRRRRRRr",
    "rRRRRRGGGGRRRRRr",
    "rRRRRRyyyyRRRRRr",
    "rRRRRRRRRRRRRRRr",
    "rRRRRRRRRRRRRRRr",
    "rRRRRRRRRRRRRRRr",
    "rrrrrrrrrrrrrrrr",
];

/// The woods a stall comes in: their planks dark to light, and their awning's two colors.
const STALL_WOODS: [(&str, [Rgba; 3], [Rgba; 2]); 3] = [
    ("oak", OAK, [[0xb3, 0x2b, 0x2b, 255], CLOTH]),
    (
        "spruce",
        [
            [0x4a, 0x33, 0x1d, 255],
            [0x6b, 0x4c, 0x2c, 255],
            [0x86, 0x63, 0x3d, 255],
        ],
        [[0x2d, 0x52, 0xa3, 255], CLOTH],
    ),
    (
        "birch",
        [
            [0xa8, 0x94, 0x62, 255],
            [0xc8, 0xb7, 0x82, 255],
            [0xde, 0xd0, 0xa0, 255],
        ],
        [[0x3e, 0x8c, 0x3a, 255], [0xe8, 0xc5, 0x3a, 255]],
    ),
];

/// What each letter means.
fn key() -> Vec<(char, Rgba)> {
    vec![
        ('d', OAK[0]),
        ('m', OAK[1]),
        ('l', OAK[2]),
        ('k', STONE[0]),
        ('s', STONE[1]),
        ('S', STONE[2]),
        ('g', GRASS[0]),
        ('G', GOLD[1]),
        ('y', GOLD[0]),
        ('Y', GOLD[2]),
        ('p', PAPER),
        ('i', INK),
        ('r', TILL[0]),
        ('R', TILL[1]),
    ]
}

/// `key` with a stall wood's letters: its planks (`D`, `M`, `L`) and awning (`a`, `b`).
fn wood_key((_, planks, cloth): (&str, [Rgba; 3], [Rgba; 2])) -> Vec<(char, Rgba)> {
    let mut letters = key();
    letters.extend([
        ('D', planks[0]),
        ('M', planks[1]),
        ('L', planks[2]),
        ('a', cloth[0]),
        ('b', cloth[1]),
    ]);
    letters
}

/// The pixels of `rows`, `scale` times as large; a letter the key lacks is clear.
fn pixels(rows: &[&str], scale: usize) -> (usize, Vec<u8>) {
    pixels_keyed(rows, scale, &key())
}

/// `pixels` with letters meaning what `key` says.
fn pixels_keyed(rows: &[&str], scale: usize, key: &[(char, Rgba)]) -> (usize, Vec<u8>) {
    let side = rows.len() * scale;
    let mut bytes = vec![0; side * side * 4];
    for (y, row) in rows.iter().enumerate() {
        for (x, letter) in row.chars().enumerate() {
            let color = key
                .iter()
                .find(|(at, _)| *at == letter)
                .map_or([0; 4], |(_, color)| *color);
            for (dy, dx) in (0..scale).flat_map(|dy| (0..scale).map(move |dx| (dy, dx))) {
                let at = ((y * scale + dy) * side + x * scale + dx) * 4;
                bytes[at..at + 4].copy_from_slice(&color);
            }
        }
    }
    (side, bytes)
}

fn png(side: usize, pixels: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut encoder = png::Encoder::new(&mut bytes, side as u32, side as u32);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("a png header is written");
    writer
        .write_image_data(pixels)
        .expect("the pixels are written");
    drop(writer);
    bytes
}

fn asset(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(path)
}

/// Writes `bytes` to `path` under `WRITE`; otherwise fails unless the file holds them already.
fn save_or_check(path: &Path, bytes: &[u8]) {
    if std::env::var_os(WRITE).is_some() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("a folder for the asset can be made");
        }
        std::fs::write(path, bytes).expect("the asset can be written");
    } else {
        let saved = std::fs::read(path).unwrap_or_default();
        assert!(
            saved == bytes,
            "{path:?} differs: {WRITE}=1 writes it again"
        );
    }
}

#[test]
fn art_drawings_are_square() {
    for (name, rows) in [
        ("plot post", &PLOT_POST),
        ("city stone", &CITY_STONE),
        ("icon", &ICON),
        ("woof", &WOOF),
        ("stall planks", &STALL_PLANKS),
        ("stall crate", &STALL_CRATE),
        ("stall post", &STALL_POST),
        ("stall awning", &STALL_AWNING),
        ("stall till", &STALL_TILL),
    ] {
        assert!(
            rows.iter().all(|row| row.chars().count() == 16),
            "{name}: every row is 16 wide"
        );
    }
}

#[test]
fn art_pictures_are_written() {
    for (path, rows, scale) in [
        ("textures/block/plot_post.png", &PLOT_POST, 1),
        ("textures/block/city_stone.png", &CITY_STONE, 1),
        ("textures/gui/icon.png", &ICON, 4),
        ("textures/gui/woof.png", &WOOF, 1),
        ("textures/block/stall_till.png", &STALL_TILL, 1),
    ] {
        let (side, pixels) = pixels(rows, scale);
        save_or_check(&asset(path), &png(side, &pixels));
    }
}

#[test]
fn art_stall_woods_are_written() {
    for wood in STALL_WOODS {
        let key = wood_key(wood);
        for (part, rows) in [
            ("planks", &STALL_PLANKS),
            ("crate", &STALL_CRATE),
            ("post", &STALL_POST),
            ("awning", &STALL_AWNING),
        ] {
            let (side, pixels) = pixels_keyed(rows, 1, &key);
            let path = format!("textures/block/{}_stall_{part}.png", wood.0);
            save_or_check(&asset(&path), &png(side, &pixels));
        }
    }
}
