//! The Hearthwood kit every building is drawn with: the material slots and their defaults, and
//! the parts the style repeats (45° roofs, timber-framed walls, windows, chimneys). See
//! `docs/materials.md` for what each slot is for.

use structgen_api::Structure;

use crate::draft::Draft;

/// One building of a family: its file name and how it is drawn.
pub type Building = (&'static str, fn() -> Structure);

/// Every slot a building may use, with its default block.
pub const SLOTS: &[(&str, &str)] = &[
    ("roof/stair", "base:spruce_planks_stairs"),
    ("roof/slab", "base:spruce_planks_slab"),
    ("roof/block", "base:spruce_planks"),
    ("frame/log", "base:oak_log"),
    ("wall/infill", "base:birch_planks"),
    ("gable", "base:spruce_planks"),
    ("plinth", "base:cobblestone"),
    ("plinth/corner", "base:stone_bricks"),
    ("plinth/stair", "base:cobblestone_stairs"),
    ("plinth/slab", "base:cobblestone_slab"),
    ("floor", "base:spruce_planks"),
    ("trim/stair", "base:spruce_planks_stairs"),
    ("trim/slab", "base:spruce_planks_slab"),
    ("window", "base:glass"),
    ("chimney", "base:bricks"),
    ("light", "base:torch"),
    ("path", "base:gravel"),
];

/// Slot `name` as a block name.
pub fn slot(name: &str) -> String {
    format!("@{name}")
}

/// A stair of `slot` whose high back is on its `facing` side.
pub fn stair(slot: &str, facing: &str, half: &str) -> String {
    format!("@{slot}[facing={facing},half={half}]")
}

/// A slab of `slot`: `bottom`, `top` or `double`.
pub fn slab(slot: &str, kind: &str) -> String {
    format!("@{slot}[type={kind}]")
}

/// A log of the frame lying along `axis` (`y` standing).
pub fn log(axis: &str) -> String {
    format!("@frame/log[axis={axis}]")
}

/// A `light` facing away from what holds it: leaning out from the wall behind it, or `up`,
/// standing on the block below.
pub fn light(facing: &str) -> String {
    format!("@light[facing={facing}]")
}

/// A `light` at `pos` (see `light`).
pub fn light_at(d: &mut Draft, pos: [usize; 3], facing: &str) {
    d.set(pos, &light(facing));
}

/// A ladder hung on the wall on its `side`. Every side is written, or a ladder keeps its
/// default plate on the north side as well.
pub fn ladder(side: &str) -> String {
    let sides = ["north", "east", "south", "west"].map(|s| format!("{s}={}", s == side));
    format!("base:ladder[{}]", sides.join(","))
}

/// A ladder up column (`x`, `z`) from height `y[0]` to `y[1]`, hung on the wall on its `side`.
pub fn ladder_up(d: &mut Draft, [x, z]: [usize; 2], y: [usize; 2], side: &str) {
    d.fill([x, y[0], z], [x, y[1], z], &ladder(side));
}

/// Which way a roof's ridge runs.
#[derive(Clone, Copy, PartialEq)]
pub enum Ridge {
    /// East to west: the slopes face north and south, the gables east and west.
    X,
    /// North to south: the gables face north and south.
    Z,
}

impl Ridge {
    /// The cell `along` the ridge, `across` it, at height `y`.
    fn at(self, along: usize, y: usize, across: usize) -> [usize; 3] {
        match self {
            Ridge::X => [along, y, across],
            Ridge::Z => [across, y, along],
        }
    }

    /// The facings of the stairs on the low and the high side across the ridge: their backs
    /// point up the slope.
    fn facings(self) -> [&'static str; 2] {
        match self {
            Ridge::X => ["south", "north"],
            Ridge::Z => ["east", "west"],
        }
    }
}

/// A 45° roof of `roof/stair` from `across[0]` to `across[1]` (eaves included), the full
/// length `along`, its eaves at height `y`. Meets in a `roof/slab` ridge when the width is odd.
/// Gives back the height of the ridge.
pub fn gable_roof(
    d: &mut Draft,
    ridge: Ridge,
    along: [usize; 2],
    across: [usize; 2],
    y: usize,
) -> usize {
    let [low, high] = ridge.facings();
    let (low_stair, high_stair) = (
        stair("roof/stair", low, "bottom"),
        stair("roof/stair", high, "bottom"),
    );
    let mut k = 0;
    while across[0] + k < across[1] - k {
        for a in along[0]..=along[1] {
            d.set(ridge.at(a, y + k, across[0] + k), &low_stair);
            d.set(ridge.at(a, y + k, across[1] - k), &high_stair);
        }
        k += 1;
    }
    if across[0] + k == across[1] - k {
        for a in along[0]..=along[1] {
            d.set(
                ridge.at(a, y + k, across[0] + k),
                &slab("roof/slab", "bottom"),
            );
        }
    }
    y + k
}

/// Fills the triangle of a gable end under a `gable_roof` of the same `across` and `y`, in the
/// wall at `along`, from `wall[0]` to `wall[1]` across, with `name`.
pub fn gable_end(
    d: &mut Draft,
    ridge: Ridge,
    along: usize,
    across: [usize; 2],
    wall: [usize; 2],
    y: usize,
    name: &str,
) {
    let mut k = 0;
    while across[0] + k < across[1] - k {
        for c in wall[0]..=wall[1] {
            if c > across[0] + k && c < across[1] - k {
                d.set(ridge.at(along, y + k, c), name);
            }
        }
        k += 1;
    }
}

/// A timber-framed wall in a straight line from `from` to `to` (x, z), heights `y[0]` to
/// `y[1]`: standing logs at both ends and every `spacing` blocks, a lying log along the top, and
/// `wall/infill` between.
pub fn timber_wall(d: &mut Draft, from: [usize; 2], to: [usize; 2], y: [usize; 2], spacing: usize) {
    let along_x = from[1] == to[1];
    let (start, end) = if along_x {
        (from[0], to[0])
    } else {
        (from[1], to[1])
    };
    let (start, end) = (start.min(end), start.max(end));
    let beam = log(if along_x { "x" } else { "z" });
    for a in start..=end {
        let [x, z] = if along_x { [a, from[1]] } else { [from[0], a] };
        let post = a == start || a == end || (a - start) % spacing == 0;
        for h in y[0]..=y[1] {
            let name = match (post, h == y[1]) {
                (true, _) => log("y"),
                (false, true) => beam.clone(),
                (false, false) => slot("wall/infill"),
            };
            d.set([x, h, z], &name);
        }
    }
}

/// A window `height` blocks tall from `pos` up, with a sill below it on the `outward` side.
pub fn window(d: &mut Draft, pos: [usize; 3], height: usize, outward: &str) {
    let [x, y, z] = pos;
    d.fill(pos, [x, y + height - 1, z], &slot("window"));
    if let Some(outside) = step([x, y - 1, z], outward) {
        d.set(outside, &stair("trim/stair", opposite(outward), "top"));
    }
}

/// A plinth of `plinth` under walls from `from` to `to` (x, z), with `plinth/corner` corners,
/// a `floor` inside, and a ring of `path` `eaves` blocks wide around it, so the ground under the
/// roof's overhang is not left open.
pub fn plinth(d: &mut Draft, from: [usize; 2], to: [usize; 2], eaves: usize) {
    let low = [from[0].saturating_sub(eaves), from[1].saturating_sub(eaves)];
    d.fill(
        [low[0], 0, low[1]],
        [to[0] + eaves, 0, to[1] + eaves],
        &slot("path"),
    );
    d.fill([from[0], 0, from[1]], [to[0], 0, to[1]], &slot("floor"));
    d.ring(0, from, to, &slot("plinth"));
    for [x, z] in [from, [to[0], from[1]], [from[0], to[1]], to] {
        d.set([x, 0, z], &slot("plinth/corner"));
    }
}

/// A chimney of `chimney` at column (`x`, `z`) from height `y[0]` to `y[1]`.
pub fn chimney(d: &mut Draft, [x, z]: [usize; 2], y: [usize; 2]) {
    d.fill([x, y[0], z], [x, y[1], z], &slot("chimney"));
}

/// The cell next to `pos` toward `facing`, `None` past the box's low edges.
pub fn step([x, y, z]: [usize; 3], facing: &str) -> Option<[usize; 3]> {
    Some(match facing {
        "north" => [x, y, z.checked_sub(1)?],
        "south" => [x, y, z + 1],
        "west" => [x.checked_sub(1)?, y, z],
        "east" => [x + 1, y, z],
        _ => panic!("no direction {facing}"),
    })
}

pub fn opposite(facing: &str) -> &'static str {
    match facing {
        "north" => "south",
        "south" => "north",
        "west" => "east",
        "east" => "west",
        _ => panic!("no direction {facing}"),
    }
}
