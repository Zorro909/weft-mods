//! Autocrafting in two steps along one row, west to east: sticks go by tube into a bench that
//! makes wooden cogs, a timber tube takes them on to a second bench, which also gets
//! cobblestone from its own line from the north and makes stone cogs into the last chest.
//!
//! Every machine runs on a spring engine, and a low wall behind the row carries redstone from a
//! lever to all of them: nothing moves until the lever is thrown on camera. The camera looks at
//! the row from the south, so nothing stands in front of it there.

use showcase_api::Set;
use weft_sdk::*;

use super::parts::{CHEST, in_world, line, spring, stock};
use crate::bench::{ASSEMBLY_BENCH, set_ghost};
use crate::lib::Pos;
use crate::tube::TIMBER_TUBE;

/// The row everything stands on, and where along it.
pub const ROW_Z: i32 = 16;
pub const STICKS: Pos = Pos::new(-12, 0, ROW_Z);
pub const STONE_COGS: Pos = Pos::new(-2, 0, ROW_Z);
const COGS: Pos = Pos::new(-7, 0, ROW_Z);
const OUTPUT: Pos = Pos::new(3, 0, ROW_Z);
/// The cobblestone's chest, north of the second bench behind the wall.
const COBBLE: Pos = Pos::new(-2, 0, ROW_Z - 3);
/// Timber tubes, each with an engine on top: out of the stick chest, out of each bench, and
/// out of the cobblestone chest.
const PULLS: [Pos; 4] = [
    Pos::new(-11, 0, ROW_Z),
    Pos::new(-6, 0, ROW_Z),
    Pos::new(-1, 0, ROW_Z),
    Pos::new(-2, 0, ROW_Z - 2),
];
/// The wall behind the row, two blocks high, from end to end of the machines.
const WALL_Z: i32 = ROW_Z - 1;
const WALL_X: [i32; 2] = [-12, -1];
/// On the wall's face above the stick chest, where the camera sees it thrown.
pub const LEVER: Pos = Pos::new(WALL_X[0], 1, ROW_Z);

/// The cells of the patterns: a wooden cog is four sticks round a gap, a stone cog four
/// cobblestones round a wooden cog.
const ROUND: [u32; 4] = [1, 3, 5, 7];
const MIDDLE: u32 = 4;

static STONE_BRICKS: Block = Block::named("base:stone_bricks");
static REDSTONE: Block = Block::named("base:redstone");
static LEVER_BLOCK: Block = Block::named("base:lever");

pub fn build(set: &Set) {
    for chest in [STICKS, COBBLE, OUTPUT] {
        set.place(chest, CHEST.id().into());
    }
    for bench in [COGS, STONE_COGS] {
        set.place(bench, ASSEMBLY_BENCH.with(&[("facing", "south")]).into());
    }
    for pull in PULLS {
        set.place(pull, TIMBER_TUBE.id().into());
    }
    for machine in [COGS, STONE_COGS].into_iter().chain(PULLS) {
        spring(set, machine.toward(Dir::Up), Dir::Down);
    }
    let on_row = |x: i32| Pos::new(x, 0, ROW_Z);
    line(set, on_row(-10), on_row(-8));
    line(set, on_row(-5), on_row(-3));
    line(set, Pos::new(-2, 0, WALL_Z), Pos::new(-2, 0, WALL_Z));
    line(set, on_row(0), on_row(2));
    power_line(set);
}

/// The wall, redstone along its top, and the lever on its face. The wire powers the wall's top
/// blocks, and so every engine touching them.
fn power_line(set: &Set) {
    let floor_wire = REDSTONE.with(&[("down", "true")]).into();
    for x in WALL_X[0]..=WALL_X[1] {
        // The cobblestone's tube runs under the wall.
        let ground = Pos::new(x, 0, WALL_Z);
        if x != COBBLE.x {
            set.place(ground, STONE_BRICKS.id().into());
        }
        set.place(ground.add_y(1), STONE_BRICKS.id().into());
        set.place(ground.add_y(2), floor_wire);
    }
    let lever = [("attach", "north"), ("front", "down"), ("power", "off")];
    set.place(LEVER, LEVER_BLOCK.with(&lever).into());
}

/// Fills the supply chests and lays out both patterns; the machines wait for the lever.
pub fn start(set: &Set) {
    stock(set, STICKS, &["base:stick"], 6);
    stock(set, COBBLE, &["base:cobblestone"], 6);
    let world = set.world();
    let item = |name: &str| Item::lookup(name).unwrap_or(0);
    let (cogs, stone_cogs) = (
        world.block_entity(in_world(set, COGS)),
        world.block_entity(in_world(set, STONE_COGS)),
    );
    for cell in ROUND {
        set_ghost(&world, cogs, cell, item("base:stick"));
        set_ghost(&world, stone_cogs, cell, item("base:cobblestone"));
    }
    set_ghost(&world, stone_cogs, MIDDLE, item("cogwork:wooden_cog"));
}

/// Throws the lever: the whole row starts.
pub fn switch_on(set: &Set) {
    set.world().prop_set(in_world(set, LEVER), "power", "on");
}
