//! Pictures of what quarries bring up and the hand machines that work it: Rubble, a block of
//! broken stone; Ore Rubble, a heap of it with flecks the game tints in the metal's color, more
//! of them the richer the grade; the sieve's string mesh and tray; and the millstone's runner.
//! The wood is base's oak planks, the stone a darker granite than the bed's smooth stone.

use super::{CLEAR, Picture, Rgba, Unwrap, noise};

/// Broken stone, dark to light: the gaps between chunks, then a chunk's shaded edge, its
/// middle and its lit edge.
pub(super) const STONE: [Rgba; 4] = [
    [0x4a, 0x4a, 0x4c, 255],
    [0x76, 0x76, 0x78, 255],
    [0x8c, 0x8c, 0x8e, 255],
    [0xa4, 0xa4, 0xa4, 255],
];
/// The flecks of Ore Rubble before the game tints them: dark to light grey, white in the light.
const FLECK: [Rgba; 3] = [
    [0x8c, 0x8c, 0x8c, 255],
    [0xc8, 0xc8, 0xc8, 255],
    [0xff, 0xff, 0xff, 255],
];
/// Oak, dark to light, as base's planks have it.
pub(super) const OAK: [Rgba; 4] = [
    [0x7e, 0x5d, 0x31, 255],
    [0x94, 0x70, 0x3f, 255],
    [0xad, 0x85, 0x50, 255],
    [0xb5, 0x8c, 0x56, 255],
];
/// String, shaded and lit.
const STRING: [Rgba; 2] = [[0xc8, 0xc8, 0xc0, 255], [0xec, 0xec, 0xe4, 255]];
/// The runner's granite, dark to light.
pub(super) const GRANITE: [Rgba; 4] = [
    [0x5c, 0x5a, 0x58, 255],
    [0x72, 0x70, 0x6c, 255],
    [0x84, 0x82, 0x7e, 255],
    [0x96, 0x94, 0x90, 255],
];
/// The flecks of each grade, Poor to Pure, and how many rocks of the heap are ore.
const FLECKS: [(&str, usize); 4] = [
    ("item/ore_rubble_poor", 2),
    ("item/ore_rubble_common", 3),
    ("item/ore_rubble_rich", 5),
    ("item/ore_rubble_pure", 7),
];
/// Ore Rubble's heap: a letter for each rock, `#` the dark gaps between them, `.` clear.
const HEAP: [&str; 16] = [
    "................",
    "................",
    "................",
    "......####......",
    ".....#aaaa#.....",
    "....#aaaaaa#....",
    "...#aaaaaa#b#...",
    "..#cc####bbbb#..",
    "..#cccc#dbbbbb#.",
    ".#ccccc#dddd###.",
    ".#cccc#ddddd#ff#",
    "#eeee#gg###ffff#",
    "#eeeee#ggg#ffff#",
    "#eeee#gggg#fff##",
    "#hhh#gggg#iiiii#",
    "################",
];
/// The rocks that turn to ore as the grade rises, in that order.
const ORE_ROCKS: &str = "dfaeicg";

/// The runner's texture and where its model's unwrap puts each box.
const RUNNER_SIZE: (usize, usize) = (64, 32);
const RUNNER_CORE: Unwrap = Unwrap {
    at: (0, 0),
    size: (8, 4, 6),
};
const RUNNER_CAPS: [Unwrap; 2] = [
    Unwrap {
        at: (0, 12),
        size: (6, 4, 1),
    },
    Unwrap {
        at: (16, 12),
        size: (6, 4, 1),
    },
];
const SPINDLE: Unwrap = Unwrap {
    at: (32, 0),
    size: (2, 2, 2),
};
const HANDLE: Unwrap = Unwrap {
    at: (40, 0),
    size: (2, 5, 2),
};
/// The tray's texture and its unwrap: four rims and the mesh.
const TRAY_SIZE: (usize, usize) = (64, 64);
const TRAY_RIMS: [Unwrap; 4] = [
    Unwrap {
        at: (0, 0),
        size: (14, 3, 1),
    },
    Unwrap {
        at: (0, 4),
        size: (14, 3, 1),
    },
    Unwrap {
        at: (30, 0),
        size: (1, 3, 12),
    },
    Unwrap {
        at: (30, 15),
        size: (1, 3, 12),
    },
];
const TRAY_MESH: Unwrap = Unwrap {
    at: (0, 32),
    size: (12, 1, 12),
};

pub fn all() -> Vec<(&'static str, Picture)> {
    let mut all = vec![
        ("block/rubble", rubble()),
        ("item/ore_rubble", heap()),
        ("block/sieve_mesh", mesh(16)),
        ("entity/millstone_runner", runner()),
        ("entity/sieve_tray", tray()),
    ];
    all.extend(FLECKS.map(|(path, count)| (path, flecks(count))));
    all
}

/// Where a pixel lies in its chunk.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Spot {
    /// On its bottom or right edge: the dark gap to the next chunk.
    Gap,
    /// On its top or left edge, which the light falls on.
    Lit,
    Inside,
}

/// A picture broken into chunks: each pixel belongs to the nearest of a grid of seeds `cell`
/// pixels apart, each moved by `noise` within its cell. Across a 16 pixel picture the grid
/// wraps round, so it tiles.
struct Chunks {
    cell: usize,
    salt: u32,
}

impl Chunks {
    /// The chunk the pixel at `x`, `y` belongs to.
    fn at(&self, x: usize, y: usize) -> usize {
        let cells = 16usize.div_ceil(self.cell);
        let wrap = |a: usize, b: usize| a.abs_diff(b).min(16 - a.abs_diff(b));
        let seed = |cell: usize| {
            let (cx, cy) = (cell % cells, cell / cells);
            let jitter = noise(cx, cy, self.salt) as usize;
            (
                cx * self.cell + jitter % self.cell,
                cy * self.cell + (jitter / 7) % self.cell,
            )
        };
        let distance = |cell: usize| {
            let (sx, sy) = seed(cell);
            wrap(x, sx).pow(2) + wrap(y, sy).pow(2)
        };
        (0..cells * cells)
            .min_by_key(|&cell| (distance(cell), cell))
            .unwrap_or(0)
    }

    /// Where the pixel at `x`, `y` lies in its chunk.
    fn spot(&self, x: usize, y: usize) -> Spot {
        let own = self.at(x, y);
        let next = |dx: usize, dy: usize| self.at((x + dx) % 16, (y + dy) % 16) != own;
        if next(1, 0) || next(0, 1) {
            Spot::Gap
        } else if next(15, 0) || next(0, 15) {
            Spot::Lit
        } else {
            Spot::Inside
        }
    }
}

/// The block's chunks.
const BLOCK_CHUNKS: Chunks = Chunks { cell: 4, salt: 101 };

/// Rubble: chunks of broken stone with dark gaps, lit from the top left.
fn rubble() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| {
        let shade = 1 + noise(BLOCK_CHUNKS.at(x, y), 0, 103) as usize % 2;
        match BLOCK_CHUNKS.spot(x, y) {
            Spot::Gap => STONE[0],
            Spot::Lit => STONE[shade + 1],
            Spot::Inside => STONE[shade],
        }
    });
    picture
}

/// The rock a pixel of the heap belongs to, by its letter in `HEAP`; none for a gap or clear.
fn rock_at(x: usize, y: usize) -> Option<u8> {
    let letter = HEAP[y].as_bytes()[x];
    letter.is_ascii_lowercase().then_some(letter)
}

/// The shade of the heap's pixel at `x`, `y` in its rock, 0 to 2: lit on its top and left
/// edges, shaded on its bottom and right ones.
fn rock_shade(x: usize, y: usize) -> usize {
    let rock = rock_at(x, y);
    let other = |x: Option<usize>, y: Option<usize>| match (x, y) {
        (Some(x), Some(y)) if x < 16 && y < 16 => rock_at(x, y) != rock,
        _ => true,
    };
    if other(x.checked_sub(1), Some(y)) || other(Some(x), y.checked_sub(1)) {
        2
    } else if other(Some(x + 1), Some(y)) || other(Some(x), Some(y + 1)) {
        0
    } else {
        1
    }
}

/// Ore Rubble's heap of grey rocks, the stone under the flecks.
fn heap() -> Picture {
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| match (HEAP[y].as_bytes()[x], rock_at(x, y)) {
        (_, Some(_)) => STONE[1 + rock_shade(x, y)],
        (b'#', None) => STONE[0],
        _ => CLEAR,
    });
    picture
}

/// The first `count` rocks of `ORE_ROCKS`, grey and lit for the game to tint, and clear
/// elsewhere, so a richer grade's flecks include every poorer grade's.
fn flecks(count: usize) -> Picture {
    let ore = &ORE_ROCKS.as_bytes()[..count];
    let mut picture = Picture::filled(CLEAR);
    picture.each(|x, y, _| match rock_at(x, y) {
        Some(rock) if ore.contains(&rock) => FLECK[rock_shade(x, y)],
        _ => CLEAR,
    });
    picture
}

/// A string mesh `size` pixels square: a string every third pixel each way, clear between.
fn mesh(size: usize) -> Picture {
    let mut picture = Picture::sized(size, size, CLEAR);
    picture.each(|x, y, _| match (x % 3 == 1, y % 3 == 1) {
        (true, true) => STRING[1],
        (true, false) | (false, true) => STRING[0],
        (false, false) => CLEAR,
    });
    picture
}

/// Oak grain: lengthwise streaks by row.
pub(super) fn oak(x: usize, y: usize) -> Rgba {
    let streak = noise(x / 4, y, 127) % 4;
    OAK[streak as usize]
}

/// The runner: speckled granite, grooves on its grinding face and its top, an oak spindle and
/// handle.
fn runner() -> Picture {
    let (width, height) = RUNNER_SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    let granite = |x: usize, y: usize| GRANITE[1 + (noise(x, y, 131) % 3) as usize];
    for stone in [RUNNER_CORE].iter().chain(&RUNNER_CAPS) {
        stone.paint(&mut picture, granite);
    }
    // Grooves across the top: every other column darker, as a dressed millstone's.
    RUNNER_CORE.paint_caps(&mut picture, |x, y, edge| {
        if edge || x % 3 == 0 {
            GRANITE[0]
        } else {
            granite(x, y)
        }
    });
    for wood in [SPINDLE, HANDLE] {
        wood.paint(&mut picture, oak);
        wood.paint_caps(
            &mut picture,
            |_, _, edge| if edge { OAK[0] } else { OAK[3] },
        );
    }
    picture
}

/// The tray: oak rims round a string mesh.
fn tray() -> Picture {
    let (width, height) = TRAY_SIZE;
    let mut picture = Picture::sized(width, height, CLEAR);
    for rim in TRAY_RIMS {
        rim.paint(&mut picture, oak);
        rim.paint_caps(
            &mut picture,
            |_, _, edge| if edge { OAK[1] } else { OAK[3] },
        );
    }
    TRAY_MESH.paint(&mut picture, oak);
    let woven = mesh(TRAY_MESH.size.0);
    TRAY_MESH.paint_caps(&mut picture, |x, y, _| {
        let (u, v) = TRAY_MESH.at;
        let d = TRAY_MESH.size.2;
        let across = (x - u - d) % TRAY_MESH.size.0;
        woven.get(across, y - v)
    });
    picture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rubble_tiles_and_has_gaps_between_its_chunks() {
        let rubble = rubble();
        let gaps = (0..16)
            .flat_map(|y| (0..16).map(move |x| (x, y)))
            .filter(|&(x, y)| rubble.get(x, y) == STONE[0])
            .count();
        assert!((40..140).contains(&gaps), "gaps, not all gap: {gaps}");
    }

    #[test]
    fn a_richer_grade_has_more_flecks_and_keeps_the_poorer_ones() {
        let lit = |picture: &Picture| {
            (0..16)
                .flat_map(|y| (0..16).map(move |x| (x, y)))
                .filter(|&(x, y)| picture.get(x, y) != CLEAR)
                .collect::<Vec<_>>()
        };
        let grades: Vec<Vec<(usize, usize)>> = FLECKS
            .iter()
            .map(|&(_, count)| lit(&flecks(count)))
            .collect();
        for pair in grades.windows(2) {
            assert!(pair[1].len() > pair[0].len(), "more flecks");
            assert!(
                pair[0].iter().all(|at| pair[1].contains(at)),
                "the poorer ones kept"
            );
        }
        assert!(!grades[0].is_empty(), "even poor shows some");
    }

    #[test]
    fn flecks_lie_on_the_heap() {
        let heap = heap();
        let pure = flecks(FLECKS[3].1);
        for y in 0..16 {
            for x in 0..16 {
                let fleck = pure.get(x, y) != CLEAR;
                assert!(
                    !fleck || heap.get(x, y) != CLEAR,
                    "a fleck off the heap at {x}, {y}"
                );
            }
        }
    }

    #[test]
    fn the_textures_match_their_models() {
        let checks = [
            (
                include_str!("../../assets/models/millstone_runner.geo.json"),
                RUNNER_SIZE,
                vec![RUNNER_CORE, RUNNER_CAPS[0], RUNNER_CAPS[1], SPINDLE, HANDLE],
            ),
            (
                include_str!("../../assets/models/sieve_tray.geo.json"),
                TRAY_SIZE,
                TRAY_RIMS.iter().copied().chain([TRAY_MESH]).collect(),
            ),
        ];
        for (model, (width, height), boxes) in checks {
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
}
