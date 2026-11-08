//! `/structure load <name> [turn]` places a structure in front of the player: the file
//! `structures/<name>.wst` that `save` writes, or else the one built into the mod.
//! `/structure undo` puts back what the player's last load replaced.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard, PoisonError};

use structures_api::{PlaceError, Replaced, Structure, Turn};
use weft_sdk::*;

use crate::{save, worldgen};

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
        return cmd.reply("structures.bad_name", &[Arg::Text(&name)]);
    }
    let Some(turn) = turn_named(cmd.text("turn").unwrap_or("none")) else {
        return cmd.reply("structures.bad_turn", &[]);
    };
    let path = save::path_of(&name);
    let file = std::fs::read(&path).ok();
    let Some(bytes) = file.as_deref().or_else(|| worldgen::embedded(&name)) else {
        let shown = path.to_string_lossy();
        return cmd.reply("structures.no_file", &[Arg::Text(&name), Arg::Text(&shown)]);
    };
    let Some(structure) = Structure::decode(bytes) else {
        return cmd.reply("structures.bad_file", &[Arg::Text(&name)]);
    };
    if structure.size().iter().any(|&side| side > save::MAX_SIDE) {
        return cmd.reply("structures.too_big", &[Arg::Int(save::MAX_SIDE as i64)]);
    }
    place(cmd, &name, &structure, turn);
}

/// Places it in front of the player, if every block it covers is loaded.
fn place(cmd: &mut ServerCommand, name: &str, structure: &Structure, turn: Turn) {
    let (world, player) = (cmd.world(), cmd.player());
    let Some(me) = world.entity(player) else {
        return cmd.reply("structures.no_player", &[]);
    };
    let size = turn.size(structure.size());
    let corner = in_front(me.pos.block().into(), facing(me.yaw), size);
    let replaced = match structure.place_in(&world, corner, turn) {
        Ok(replaced) => replaced,
        Err(PlaceError::UnknownBlock(block)) => {
            let args = [Arg::Text(name), Arg::Text(&block)];
            return cmd.reply("structures.unknown_block", &args);
        }
        Err(PlaceError::NotLoaded) => return cmd.reply("structures.not_loaded", &[]),
    };
    let undo = Undo {
        dimension: world.dimension(),
        replaced,
    };
    lock().insert(player, undo);
    let [x, y, z] = corner.map(|v| Arg::Int(v as i64));
    cmd.reply("structures.loaded", &[Arg::Text(name), x, y, z]);
}

/// `/structure undo`.
pub fn undo(cmd: &mut ServerCommand) {
    let (world, player) = (cmd.world(), cmd.player());
    let Some(undo) = take_undo(player, world.dimension()) else {
        return cmd.reply("structures.nothing_to_undo", &[]);
    };
    structures_api::put_back(&world, &undo.replaced);
    let count = Arg::Int(undo.replaced.len() as i64);
    cmd.reply("structures.undone", &[count]);
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

/// The way `yaw` (radians) looks, as a step along x or z.
fn facing(yaw: f32) -> [i32; 2] {
    let (x, z) = (-(yaw as f64).sin(), -(yaw as f64).cos());
    if x.abs() > z.abs() {
        [x.signum() as i32, 0]
    } else {
        [0, z.signum() as i32]
    }
}

/// The lowest corner that puts a box of `size` `GAP` blocks in front of `feet` along `facing`,
/// centred on the player, its bottom layer taking the place of the block they stand on.
fn in_front(feet: [i32; 3], facing: [i32; 2], [sx, _, sz]: [i32; 3]) -> [i32; 3] {
    let along = |feet: i32, step: i32, side: i32| match step {
        1 => feet + GAP,
        -1 => feet - GAP - side + 1,
        _ => feet - side / 2,
    };
    [
        along(feet[0], facing[0], sx),
        feet[1] - 1,
        along(feet[2], facing[1], sz),
    ]
}

fn lock() -> MutexGuard<'static, HashMap<u64, Undo>> {
    UNDOS.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::f32::consts::FRAC_PI_2;

    use super::*;

    #[test]
    fn yaw_turns_into_a_step() {
        assert_eq!(facing(0.0), [0, -1], "north at 0");
        assert_eq!(facing(FRAC_PI_2), [-1, 0], "west a quarter turn on");
        assert_eq!(facing(-FRAC_PI_2), [1, 0], "east");
        assert_eq!(facing(3.0), [0, 1], "south");
    }

    #[test]
    fn the_structure_goes_in_front() {
        let size = [5, 3, 4];
        let feet = [10, 64, 10];
        assert_eq!(
            in_front(feet, [0, -1], size),
            [8, 63, 5],
            "north: ends two blocks ahead"
        );
        assert_eq!(
            in_front(feet, [1, 0], size),
            [12, 63, 8],
            "east: starts two blocks ahead"
        );
        assert_eq!(
            in_front(feet, [0, 1], size)[1],
            63,
            "its floor where the player stands"
        );
    }

    #[test]
    fn an_undo_waits_in_its_own_dimension() {
        let replaced = vec![([0, 64, 0], vec![1])];
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
