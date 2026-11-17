//! Pictures of the acid works and the seismic thumper: sulfuric acid, still and in its bucket,
//! the acid tank's sight glass, the thumper's strike plate and gearbox, and its weight. The
//! weight's texture follows its Blockbench model's boxes (`Unwrap`).

use super::steel_items::bucket_of;
use super::{CLEAR, GLINT, Picture, Rgba, STEEL, Unwrap, noise};

/// Sulfuric acid: a pale, oily yellow-green, dark to its sheen.
const ACID: [Rgba; 5] = [
    [0x6a, 0x72, 0x1c, 255],
    [0x96, 0xa2, 0x2a, 255],
    [0xb8, 0xc4, 0x3e, 255],
    [0xd6, 0xde, 0x6a, 255],
    [0xf2, 0xf6, 0xb0, 255],
];
/// How opaque acid is: see-through, as water.
const ACID_ALPHA: u8 = 0xc8;
/// Frames of the acid's bubbling, each shown for its `.anim`'s frametime.
const FRAMES: usize = 8;
/// Bubbles rising through the acid at once.
const BUBBLES: usize = 6;
/// The empty glass above the acid in a sight glass.
const GLASS: Rgba = [0x3a, 0x48, 0x52, 255];
/// The warning stripes round the strike plate.
const WARNING: [Rgba; 2] = [[0xe0, 0xb0, 0x20, 255], [0x1c, 0x1c, 0x1c, 255]];
/// Cream plastic, as the plastic works have it: the gears in the thumper's gearbox.
const PLASTIC: [Rgba; 3] = [
    [0x8a, 0x84, 0x70, 255],
    [0xd8, 0xd0, 0xb8, 255],
    [0xf2, 0xee, 0xde, 255],
];
/// The rows of the acid tank's sight glass, top and bottom, and where its acid stands.
const GLASS_ROWS: [usize; 2] = [5, 10];
const GLASS_COLUMNS: [usize; 2] = [6, 9];
const ACID_FROM: usize = 7;
/// The weight's boxes: the weight, and the rod it hangs from.
const WEIGHT: Unwrap = Unwrap::new((0, 0), (8, 5, 8));
const WEIGHT_ROD: Unwrap = Unwrap::new((32, 0), (2, 9, 2));

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/sulfuric_acid_still", acid_still()),
        ("item/sulfuric_acid_bucket", bucket_of(&ACID)),
        ("block/acid_tank_glass", sight_glass()),
        ("block/thumper_plate", strike_plate()),
        ("block/thumper_gearbox", gearbox()),
        ("entity/seismic_thumper_weight", weight()),
    ]
}

/// Acid: pale and see-through, with small bubbles rising and bursting, so a vat of it fizzes.
fn acid_still() -> Picture {
    let frames: Vec<Picture> = (0..FRAMES).map(acid_frame).collect();
    Picture::stacked(&frames)
}

/// One frame of the acid: its surface with each bubble a little further on, a light dot that
/// bursts into a ring. They wrap round the edges, so the acid tiles.
fn acid_frame(frame: usize) -> Picture {
    let mut picture = Picture::surface(&ACID[1..4], 911);
    picture.each(|x, y, color| {
        let mut color = color;
        for bubble in 0..BUBBLES {
            let at = |axis: usize| noise(bubble, axis, 919) as usize % 16;
            let age = (noise(bubble, 2, 919) as usize + frame) % FRAMES;
            let (cx, cy) = (at(0), (at(1) + 16 - age) % 16);
            let (dx, dy) = (
                x.abs_diff(cx).min(16 - x.abs_diff(cx)),
                y.abs_diff(cy).min(16 - y.abs_diff(cy)),
            );
            color = match (age, dx + dy) {
                (6, 1) | (7, 2) => ACID[0],
                (0..=5, 0) => ACID[4],
                _ => continue,
            };
        }
        let [r, g, b, _] = color;
        [r, g, b, ACID_ALPHA]
    });
    picture
}

/// The acid tank's sight glass: a steel frame round a window, acid in its lower half under dark
/// empty glass, with a glint.
fn sight_glass() -> Picture {
    let mut picture = Picture::filled(STEEL[2]);
    picture.each(|x, y, _| {
        let rim = GLASS_COLUMNS.contains(&x) || GLASS_ROWS.contains(&y);
        let inside = (GLASS_COLUMNS[0]..=GLASS_COLUMNS[1]).contains(&x)
            && (GLASS_ROWS[0]..=GLASS_ROWS[1]).contains(&y);
        match (rim, inside) {
            (true, _) | (_, false) => STEEL[1],
            _ if y == ACID_FROM => ACID[3],
            _ if y > ACID_FROM => ACID[2 - (noise(x, y, 929) % 2) as usize],
            _ if (x, y) == (GLASS_COLUMNS[0] + 1, GLASS_ROWS[0] + 1) => GLINT,
            _ => GLASS,
        }
    });
    picture
}

/// The thumper's strike plate: worn dark steel inside a border of warning stripes.
fn strike_plate() -> Picture {
    let mut picture = Picture::filled(STEEL[1]);
    picture.each(|x, y, _| {
        let border = x < 5 || y < 5 || x > 10 || y > 10;
        match border {
            true if (x + y) % 4 < 2 => WARNING[0],
            true => WARNING[1],
            false if noise(x, y, 937).is_multiple_of(5) => STEEL[3],
            false => STEEL[2],
        }
    });
    picture
}

/// A face of the thumper's gearbox: a window in its steel housing onto a cream plastic gear of
/// eight teeth, with a second one meshing below. The window fills the face the model shows,
/// texels 3 to 12 across and 4 to 13 down; the commas around it are its housing.
fn gearbox() -> Picture {
    let rows = [
        ",,,,,,,,,,,,,,,,",
        ",,,,,,,,,,,,,,,,",
        ",,,,,,,,,,,,,,,,",
        ",,,,,,,,,,,,,,,,",
        ",,,ssssssssss,,,",
        ",,,s..g..g..s,,,",
        ",,,s.gGGGGg.s,,,",
        ",,,sgGGddGGgs,,,",
        ",,,s.Gd..dG.s,,,",
        ",,,s.Gd..dG.s,,,",
        ",,,sgGGddGGgs,,,",
        ",,,s.gGGGGg.s,,,",
        ",,,s.dgdgdgds,,,",
        ",,,ssssssssss,,,",
        ",,,,,,,,,,,,,,,,",
        ",,,,,,,,,,,,,,,,",
    ];
    let key = [
        ('s', STEEL[0]),
        ('.', STEEL[1]),
        ('g', PLASTIC[1]),
        ('G', PLASTIC[2]),
        ('d', PLASTIC[0]),
    ];
    let mut picture = Picture::drawn(&rows, &key);
    picture.each(|_, _, color| if color == CLEAR { STEEL[2] } else { color });
    picture
}

/// The thumper's weight: riveted dark steel with a band of warning stripes round its foot, on a
/// bright steel rod.
fn weight() -> Picture {
    let mut picture = Picture::sized(64, 32, CLEAR);
    WEIGHT.paint(&mut picture, |x, y| {
        let side = y >= 8;
        let foot = side && y >= 8 + 3;
        match () {
            _ if foot && (x + y) % 4 < 2 => WARNING[0],
            _ if foot => WARNING[1],
            _ if side && y == 9 && x % 4 == 1 => STEEL[4],
            _ if noise(x, y, 941).is_multiple_of(6) => STEEL[1],
            _ => STEEL[2],
        }
    });
    WEIGHT_ROD.paint(
        &mut picture,
        |x, _| if x % 2 == 0 { STEEL[4] } else { STEEL[3] },
    );
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acid_bubbles_from_frame_to_frame() {
        let (first, next) = (acid_frame(0), acid_frame(1));
        let differs = (0..16).any(|y| (0..16).any(|x| first.get(x, y) != next.get(x, y)));
        assert!(differs, "it moves");
        assert_eq!(first.get(3, 3)[3], ACID_ALPHA, "see-through");
    }

    #[test]
    fn the_sight_glass_shows_acid_under_empty_glass() {
        let glass = sight_glass();
        assert_eq!(glass.get(8, 6), GLASS, "empty above");
        assert!(ACID.contains(&glass.get(7, 9)), "acid below");
        assert_eq!(glass.get(6, 8), STEEL[1], "in a steel frame");
    }

    #[test]
    fn the_weight_fills_its_unwraps() {
        let weight = weight();
        assert_ne!(weight.get(15, 0)[3], 0, "the weight's top");
        assert_ne!(weight.get(0, 12)[3], 0, "its foot");
        assert_ne!(weight.get(35, 10)[3], 0, "the rod's last row");
    }
}
