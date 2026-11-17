//! Pictures of the bellows' moving parts: a bag of leather pleats, each lit along its top and
//! creased dark along its bottom, under an oak lid; the hand bellows' oak handle, and the driven
//! bellows' iron push rod and cogs. The leather is also a block texture, for the items.

use super::processing::{OAK, oak};
use super::{CLEAR, IRON, Picture, Rgba, Unwrap, noise};

/// Leather, dark to light: a crease, its shade, its middle and where the light catches it.
const LEATHER: [Rgba; 4] = [
    [0x3e, 0x24, 0x16, 255],
    [0x5c, 0x37, 0x20, 255],
    [0x74, 0x47, 0x2a, 255],
    [0x8e, 0x5a, 0x36, 255],
];
/// Each bag model's texture size, and where its unwrap puts each box.
const SIZE: (usize, usize) = (64, 64);
const PLEATS: [Unwrap; 3] = [
    Unwrap {
        at: (0, 0),
        size: (10, 2, 10),
    },
    Unwrap {
        at: (0, 12),
        size: (12, 2, 11),
    },
    Unwrap {
        at: (0, 25),
        size: (10, 2, 10),
    },
];
const LID: Unwrap = Unwrap {
    at: (0, 37),
    size: (12, 2, 13),
};
/// The hand bellows' handle: a post and a grip across it.
const HANDLE: [Unwrap; 2] = [
    Unwrap {
        at: (0, 52),
        size: (2, 3, 2),
    },
    Unwrap {
        at: (10, 52),
        size: (8, 2, 2),
    },
];
/// The driven bellows' push rod, and a cog's spoke and arm: every cog shows the same.
const ROD: Unwrap = Unwrap {
    at: (0, 52),
    size: (2, 5, 2),
};
const COG: [Unwrap; 2] = [
    Unwrap {
        at: (10, 52),
        size: (1, 6, 2),
    },
    Unwrap {
        at: (18, 52),
        size: (1, 2, 2),
    },
];

pub fn all() -> Vec<(&'static str, Picture)> {
    vec![
        ("block/bellows_leather", leather_block()),
        ("entity/hand_bellows_bag", hand_bag()),
        ("entity/driven_bellows_bag", driven_bag()),
    ]
}

/// Leather, mottled.
fn leather(x: usize, y: usize) -> Rgba {
    LEATHER[1 + (noise(x, y, 173) % 3) as usize]
}

/// Leather pleats two pixels high, as the items' boxes put them: the light on each even row,
/// the crease on each odd one.
fn leather_block() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| pleat_row(x, y, y % 2 == 0));
    picture
}

/// A pixel of a pleat's side: lit along its `top` row, creased dark along the other.
fn pleat_row(x: usize, y: usize, top: bool) -> Rgba {
    if top {
        if noise(x, y, 179).is_multiple_of(3) {
            LEATHER[2]
        } else {
            LEATHER[3]
        }
    } else {
        LEATHER[0]
    }
}

/// The bag and the lid every bellows has, on a clear picture.
fn bag() -> Picture {
    let (width, height) = SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    for pleat in PLEATS {
        let top_row = pleat.at.1 + pleat.size.2;
        pleat.paint(&mut picture, |x, y| pleat_row(x, y, y == top_row));
        pleat.paint_caps(&mut picture, |x, y, _| leather(x, y));
    }
    LID.paint(&mut picture, oak);
    LID.paint_caps(
        &mut picture,
        |x, y, edge| if edge { OAK[0] } else { oak(x, y) },
    );
    picture
}

/// The hand bellows' bag, lid and a darker, worn oak handle.
fn hand_bag() -> Picture {
    let mut picture = bag();
    for part in HANDLE {
        part.paint(&mut picture, |x, y| OAK[(noise(x, y, 181) % 2) as usize]);
    }
    picture
}

/// The driven bellows' bag and lid, an iron push rod and iron cogs with dark edges.
fn driven_bag() -> Picture {
    let mut picture = bag();
    ROD.paint(&mut picture, |x, y| {
        IRON[2 + (noise(x, y, 191) % 2) as usize]
    });
    for part in COG {
        part.paint(&mut picture, |_, _| IRON[2]);
        part.paint_ends(
            &mut picture,
            |_, _, edge| if edge { IRON[1] } else { IRON[3] },
        );
    }
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_textures_match_their_models() {
        let hand = [PLEATS.as_slice(), &[LID], &HANDLE].concat();
        let driven = [PLEATS.as_slice(), &[LID, ROD], &COG].concat();
        let checks = [
            (
                include_str!("../../assets/models/hand_bellows_bag.geo.json"),
                hand,
            ),
            (
                include_str!("../../assets/models/driven_bellows_bag.geo.json"),
                driven,
            ),
        ];
        let (width, height) = SIZE;
        for (model, boxes) in checks {
            for unwrap in boxes {
                let (w, h, d) = unwrap.size;
                let (u, v) = unwrap.at;
                let cube = format!("\"size\": [{w}, {h}, {d}], \"uv\": [{u}, {v}]");
                assert!(model.contains(&cube), "a box {cube}");
            }
            assert!(
                model.contains(&format!("\"texture_width\": {width}")),
                "as wide"
            );
            assert!(
                model.contains(&format!("\"texture_height\": {height}")),
                "as high"
            );
        }
    }

    #[test]
    fn every_pleat_is_lit_on_top_and_creased_below() {
        let picture = leather_block();
        for y in 0..16 {
            let creased = (0..16).all(|x| picture.get(x, y) == LEATHER[0]);
            assert_eq!(creased, y % 2 == 1, "row {y}");
        }
    }
}
