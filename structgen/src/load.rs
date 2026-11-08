//! `/structure load <name> [turn] [styles]` places a structure in front of the player: the file
//! `structures/<name>.wst` that `save` writes, or else the one built into the mod, its material
//! slots filled with the styles named (`brick_roof,spruce_frame`).
//! `/structure undo` puts back what the player's last load replaced.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

use structgen_api::{Materials, PlaceError, Pos, Replaced, Structure, Turn};
use weft_sdk::*;

use crate::{save, styles, worldgen};

/// How many blocks ahead of the player the structure starts.
const GAP: i32 = 2;

/// What each player's last load replaced, by player entity.
static UNDOS: LazyLock<Mutex<HashMap<u64, Undo>>> = LazyLock::new(Default::default);

struct Undo {
    dimension: u32,
    replaced: Replaced,
}

pub fn run(cmd: &mut ServerCommand) {
    let Some(name) = cmd.text("file").map(str::to_string) else {
        return;
    };
    if !save::valid_name(&name) {
        return cmd.reply("structgen.bad_name", &[Arg::Text(&name)]);
    }
    let Some(turn) = turn_named(cmd.text("turn").unwrap_or("none")) else {
        return cmd.reply("structgen.bad_turn", &[]);
    };
    let Some(materials) = styles::named(cmd.text("styles").unwrap_or("")) else {
        let all: Vec<&str> = styles::STYLES.iter().map(|(name, _)| *name).collect();
        return cmd.reply("structgen.bad_style", &[Arg::Text(&all.join(", "))]);
    };
    let path = save::path_of(&name);
    let file = std::fs::read(&path).ok();
    let Some(bytes) = file.as_deref().or_else(|| worldgen::embedded(&name)) else {
        let shown = path.to_string_lossy();
        return cmd.reply("structgen.no_file", &[Arg::Text(&name), Arg::Text(&shown)]);
    };
    let Some(structure) = Structure::decode(bytes) else {
        return cmd.reply("structgen.bad_file", &[Arg::Text(&name)]);
    };
    if structure.size().iter().any(|&side| side > save::MAX_SIDE) {
        return cmd.reply("structgen.too_big", &[Arg::Int(save::MAX_SIDE as i64)]);
    }
    place(cmd, &name, &structure, turn, &materials);
}

/// Places it in front of the player, if every block it covers is loaded.
fn place(
    cmd: &mut ServerCommand,
    name: &str,
    structure: &Structure,
    turn: Turn,
    materials: &Materials,
) {
    let (world, player) = (cmd.world(), cmd.player());
    let Some(me) = world.entity(player) else {
        return cmd.reply("structgen.no_player", &[]);
    };
    let size = turn.size(structure.size());
    let corner = in_front(me.pos.block().into(), facing(me.yaw), size);
    let replaced = match structure.place_in(&world, corner, turn, materials) {
        Ok(replaced) => replaced,
        Err(PlaceError::UnknownBlock(block)) => {
            let args = [Arg::Text(name), Arg::Text(&block)];
            return cmd.reply("structgen.unknown_block", &args);
        }
        Err(PlaceError::NotLoaded) => return cmd.reply("structgen.not_loaded", &[]),
    };
    let undo = Undo {
        dimension: world.dimension(),
        replaced,
    };
    lock().insert(player, undo);
    let [x, y, z] = <[i32; 3]>::from(corner).map(|v| Arg::Int(v as i64));
    cmd.reply("structgen.loaded", &[Arg::Text(name), x, y, z]);
}

/// `/structure undo`.
pub fn undo(cmd: &mut ServerCommand) {
    let (world, player) = (cmd.world(), cmd.player());
    let Some(undo) = take_undo(player, world.dimension()) else {
        return cmd.reply("structgen.nothing_to_undo", &[]);
    };
    structgen_api::put_back(&world, &undo.replaced);
    let count = Arg::Int(undo.replaced.len() as i64);
    cmd.reply("structgen.undone", &[count]);
}

/// What `player`'s last load replaced, if that was in `dimension`. Kept for later otherwise.
fn take_undo(player: u64, dimension: u32) -> Option<Undo> {
    let mut all = lock();
    let here = all.get(&player)?.dimension == dimension;
    if here { all.remove(&player) } else { None }
}

fn turn_named(name: &str) -> Option<Turn> {
    let names = ["none", "right", "back", "left"];
    names.iter().position(|&n| n == name).map(|i| Turn::ALL[i])
}

/// The horizontal direction nearest the way `yaw` (radians) looks.
fn facing(yaw: f32) -> Dir {
    let [x, z] = yaw_dir(yaw);
    Dir::nearest([x, 0.0, z], Facings::HORIZONTAL)
}

/// The lowest corner that puts a box of `size` `GAP` blocks in front of `feet` toward `facing`,
/// centred on the player, its bottom layer taking the place of the block they stand on.
fn in_front(feet: Pos, facing: Dir, [sx, _, sz]: [i32; 3]) -> Pos {
    let step = facing.offset();
    let along = |feet: i32, step: i32, side: i32| match step {
        1 => feet + GAP,
        -1 => feet - GAP - side + 1,
        _ => feet - side / 2,
    };
    Pos::new(
        along(feet.x, step.x, sx),
        feet.y - 1,
        along(feet.z, step.z, sz),
    )
}

fn lock() -> MutexGuard<'static, HashMap<u64, Undo>> {
    UNDOS.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use structgen_api::BlockState;

    use super::*;

    #[test]
    fn yaw_turns_into_a_step() {
        assert_eq!(facing(0.0), Dir::North, "north at 0");
        assert_eq!(facing(FRAC_PI_2), Dir::West, "west a quarter turn on");
        assert_eq!(facing(-FRAC_PI_2), Dir::East, "east");
        assert_eq!(facing(3.0), Dir::South, "south");
    }

    #[test]
    fn the_structure_goes_in_front() {
        let size = [5, 3, 4];
        let feet = Pos::new(10, 64, 10);
        assert_eq!(
            in_front(feet, Dir::North, size),
            Pos::new(8, 63, 5),
            "north: ends two blocks ahead"
        );
        assert_eq!(
            in_front(feet, Dir::East, size),
            Pos::new(12, 63, 8),
            "east: starts two blocks ahead"
        );
        assert_eq!(
            in_front(feet, Dir::South, size).y,
            63,
            "its floor where the player stands"
        );
    }

    #[test]
    fn an_undo_waits_in_its_own_dimension() {
        let replaced = vec![(Pos::new(0, 64, 0), vec![BlockState::from(1)])];
        let undo = Undo {
            dimension: 2,
            replaced,
        };
        lock().insert(77, undo);
        assert!(
            take_undo(77, 3).is_none(),
            "nothing to undo in another dimension"
        );
        assert!(take_undo(77, 2).is_some(), "still there back home");
        assert!(take_undo(77, 2).is_none(), "and undone once");
    }

    #[test]
    fn turns_are_named() {
        assert_eq!(turn_named("back"), Some(Turn::Back), "a name");
        assert_eq!(turn_named("sideways"), None, "anything else");
    }
}
