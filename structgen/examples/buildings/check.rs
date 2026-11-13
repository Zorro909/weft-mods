//! What is wrong with a drawn structure, found from its cells alone before the game is started
//! for its pictures: holes dug under it, blocks hanging in the air, and furniture nobody can walk
//! to. A walker stands in for the player; what it can't judge (the look, the light, a stair's
//! facing) stays with the pictures.

use std::collections::HashSet;

use crate::cells::{Cell, Cells, LADDER};
use crate::draft::AIR;
use crate::structure_file::Structure;

/// Blocks a player uses, so has to get next to.
const FURNITURE: &[&str] = &[
    "base:chest",
    "base:furnace",
    "base:crafting_table",
    "base:woodcutter",
    "base:bed",
];
/// The values the game knows for the properties the kit writes. A value it doesn't know is
/// dropped without a word, and the block stands as its default.
const VALUES: &[(&str, &[&str])] = &[
    ("facing", &["north", "east", "south", "west", "up"]),
    ("half", &["bottom", "top"]),
    ("type", &["bottom", "top", "double"]),
    ("axis", &["x", "y", "z"]),
    (
        "shape",
        &[
            "straight",
            "inner_left",
            "inner_right",
            "outer_left",
            "outer_right",
        ],
    ),
];
/// The four ways a walker steps.
const SIDES: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
/// The height a walker's feet are at on the ground around a structure: on its bottom layer.
const GROUND_FEET: i32 = 1;

/// Every fault of `structure`, as sentences naming the cell.
pub fn faults(structure: &Structure) -> Vec<String> {
    let cells = Cells::of(structure);
    let mut found = misspelt(structure);
    found.extend(holes(structure));
    found.extend(floating(structure, &cells));
    found.extend(out_of_reach(structure, &walked(&cells)));
    found
}

/// Blocks with a property value the game doesn't know, each kind once.
fn misspelt(structure: &Structure) -> Vec<String> {
    let mut found: Vec<String> = (structure.blocks())
        .filter(|(_, name)| properties(name).any(|(key, value)| !known(key, value)))
        .map(|(_, name)| format!("{name} has a property value the game doesn't know"))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// The `key=value` pairs between a name's brackets.
fn properties(name: &str) -> impl Iterator<Item = (&str, &str)> {
    let inside = name.split_once('[').map_or("", |(_, rest)| rest);
    inside
        .trim_end_matches(']')
        .split(',')
        .filter_map(|pair| pair.split_once('='))
}

/// Whether `value` is one the game knows for `key`; any value of a property not listed here.
fn known(key: &str, value: &str) -> bool {
    VALUES
        .iter()
        .find(|(listed, _)| *listed == key)
        .is_none_or(|(_, values)| values.contains(&value))
}

/// Air in the bottom layer, which digs the ground out where the structure is placed.
fn holes(structure: &Structure) -> Vec<String> {
    let dug: Vec<Cell> = (structure.blocks())
        .filter(|&(at, name)| at[1] == 0 && name == AIR)
        .map(|(at, _)| at)
        .collect();
    let Some(first) = dug.first() else {
        return Vec::new();
    };
    vec![format!(
        "{} cells of the bottom layer are air, {first:?} the first: that digs a hole; give \
         them the ground or a path slot",
        dug.len()
    )]
}

/// Blocks that touch nothing, not even by a corner, that leads down to the bottom layer.
fn floating(structure: &Structure, cells: &Cells) -> Vec<String> {
    let solid = |at: Cell| cells.holds(at) && cells.block(at) != AIR;
    let grounded = spread(
        (structure.blocks())
            .filter(|&(at, name)| at[1] == 0 && name != AIR)
            .map(|(at, _)| at),
        |at| around(at).into_iter().filter(|&next| solid(next)).collect(),
    );
    let mut adrift: Vec<Cell> = (structure.blocks())
        .filter(|&(at, name)| name != AIR && !grounded.contains(&at))
        .map(|(at, _)| at)
        .collect();
    adrift.sort();
    let Some(&first) = adrift.first() else {
        return Vec::new();
    };
    vec![format!(
        "{} blocks hang in the air, {} at {first:?} the first",
        adrift.len(),
        cells.block(first)
    )]
}

/// The 26 cells around `at`.
fn around([x, y, z]: Cell) -> Vec<Cell> {
    let offsets = [-1, 0, 1];
    let mut cells = Vec::new();
    for dx in offsets {
        for dy in offsets {
            for dz in offsets {
                if [dx, dy, dz] != [0, 0, 0] {
                    cells.push([x + dx, y + dy, z + dz]);
                }
            }
        }
    }
    cells
}

/// Every cell a walker's feet get to from the ground around the structure: stepping sideways,
/// one block up or down as it goes, and up and down ladders.
fn walked(cells: &Cells) -> HashSet<Cell> {
    let [sx, _, sz] = cells.size;
    // The ground one block out from the box, all the way round.
    let outside = |[x, y, z]: Cell| {
        let ring = x == -1 || x == sx || z == -1 || z == sz;
        y == GROUND_FEET && ring && (-1..=sx).contains(&x) && (-1..=sz).contains(&z)
    };
    let start = (-1..=sx).map(|x| [x, GROUND_FEET, -1]);
    spread(start, |[x, y, z]| {
        let mut next = Vec::new();
        for [dx, dz] in SIDES {
            for dy in [0, 1, -1] {
                next.push([x + dx, y + dy, z + dz]);
            }
        }
        let on_ladder = |at: Cell| cells.block(at) == LADDER;
        for dy in [1, -1] {
            let climbed = [x, y + dy, z];
            if (on_ladder([x, y, z]) || on_ladder(climbed)) && cells.open(climbed) {
                next.push(climbed);
            }
        }
        next.retain(|&at| outside(at) || (cells.holds(at) && cells.standable(at)));
        next
    })
}

/// Every cell reached from `start` by going on to what `next` gives for each.
fn spread(start: impl Iterator<Item = Cell>, next: impl Fn(Cell) -> Vec<Cell>) -> HashSet<Cell> {
    let mut reached: HashSet<Cell> = start.collect();
    let mut waiting: Vec<Cell> = reached.iter().copied().collect();
    while let Some(at) = waiting.pop() {
        for cell in next(at) {
            if reached.insert(cell) {
                waiting.push(cell);
            }
        }
    }
    reached
}

/// Furniture with no walked cell beside it, at its own height or one above or below.
fn out_of_reach(structure: &Structure, walked: &HashSet<Cell>) -> Vec<String> {
    let beside = |[x, y, z]: Cell| {
        SIDES.iter().any(|[dx, dz]| {
            [-1, 0, 1]
                .iter()
                .any(|dy| walked.contains(&[x + dx, y + dy, z + dz]))
        })
    };
    let mut found: Vec<String> = (structure.blocks())
        .filter(|&(at, name)| is_furniture(name) && !beside(at))
        .map(|(at, name)| {
            format!("no way in from outside leads to the {name} at {at:?}: a doorway is shut, or a floor has no ladder")
        })
        .collect();
    found.sort();
    found
}

fn is_furniture(name: &str) -> bool {
    FURNITURE.contains(&name.split('[').next().unwrap_or(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draft::Draft;

    /// A hut 5 by 5 with walls 3 high, a roof, a chest inside and, if `door`, a doorway north.
    fn hut(door: bool) -> Draft {
        let mut hut = Draft::new([5, 5, 5]);
        hut.fill([0, 0, 0], [4, 0, 4], "base:grass");
        for y in 1..=3 {
            hut.ring(y, [0, 0], [4, 4], "base:oak_planks");
        }
        hut.fill([0, 4, 0], [4, 4, 4], "base:oak_planks");
        hut.set([2, 1, 3], "base:chest[facing=north]");
        if door {
            hut.fill([2, 1, 0], [2, 2, 0], AIR);
        }
        hut
    }

    #[test]
    fn a_hut_with_a_doorway_has_no_fault() {
        assert_eq!(faults(&hut(true).done()), Vec::<String>::new());
    }

    #[test]
    fn a_chest_behind_walls_is_out_of_reach() {
        let found = faults(&hut(false).done());
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(
            found[0].contains("base:chest[facing=north] at [2, 1, 3]"),
            "{found:?}"
        );
    }

    #[test]
    fn a_misspelt_property_value_is_named_once() {
        let mut odd = hut(true);
        odd.fill(
            [1, 4, 1],
            [2, 4, 1],
            "@roof/stair[facing=nroth,half=bottom]",
        );
        odd.set([3, 4, 1], "base:wheat[age=7]");
        let found = faults(&odd.done());
        let expected =
            "@roof/stair[facing=nroth,half=bottom] has a property value the game doesn't know";
        assert_eq!(found, [expected]);
    }

    #[test]
    fn a_doorway_one_block_high_lets_nobody_in() {
        let mut low = hut(true);
        low.set([2, 2, 0], "base:oak_planks");
        assert_eq!(faults(&low.done()).len(), 1, "the chest stays out of reach");
    }

    /// A tower with a chest on a floor four blocks up and, if `ladder`, a ladder to it.
    fn loft(ladder: bool) -> Draft {
        let mut loft = Draft::new([5, 9, 5]);
        loft.fill([0, 0, 0], [4, 0, 4], "base:grass");
        for y in 1..=7 {
            loft.ring(y, [0, 0], [4, 4], "base:oak_planks");
        }
        loft.fill([2, 1, 0], [2, 2, 0], AIR);
        loft.fill([1, 4, 1], [3, 4, 3], "base:oak_planks");
        loft.set([2, 5, 2], "base:chest[facing=north]");
        if ladder {
            loft.fill([1, 1, 3], [1, 4, 3], "base:ladder[south=true]");
        }
        loft
    }

    #[test]
    fn a_ladder_leads_to_the_floor_above() {
        assert_eq!(faults(&loft(false).done()).len(), 1, "no way up yet");
        assert_eq!(faults(&loft(true).done()), Vec::<String>::new());
    }

    #[test]
    fn air_in_the_bottom_layer_is_a_hole() {
        let mut dug = hut(true);
        dug.fill([1, 0, 1], [2, 0, 1], AIR);
        let found = faults(&dug.done());
        assert!(
            found[0].starts_with("2 cells of the bottom layer are air, [1, 0, 1]"),
            "{found:?}"
        );
    }

    /// A log with a plank at its upper corner and, if `adrift`, glass well above both.
    fn corner(adrift: bool) -> Draft {
        let mut odd = Draft::new([5, 9, 5]);
        odd.fill([0, 0, 0], [4, 0, 4], "base:grass");
        odd.set([2, 1, 2], "base:oak_log");
        odd.set([3, 2, 3], "base:oak_planks");
        if adrift {
            odd.set([2, 6, 2], "base:glass");
        }
        odd
    }

    #[test]
    fn a_block_touching_nothing_hangs_in_the_air() {
        assert_eq!(
            faults(&corner(false).done()),
            Vec::<String>::new(),
            "a corner holds it"
        );
        let found = faults(&corner(true).done());
        assert_eq!(
            found,
            ["1 blocks hang in the air, base:glass at [2, 6, 2] the first"]
        );
    }
}
