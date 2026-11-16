//! The spawn: where a game master stood for `/setspawn`, protected in a circle around it. The
//! engine's own spawn point is fixed when the world loads, so a player new to the server lands
//! there first and is moved here before they see it; so is one respawning with no bed (base
//! moves those with one).

use base_api::BED_SPAWN;
use serde::{Deserialize, Serialize};
use weft_sdk::{Arg, PLAYER_RESPAWNED, Registry, ServerCommand, World, permission};
use weftkit::Pos;

use crate::{dims, settings, tables::SPAWN};

/// Where players start, and how far around it is protected.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub struct Spawn {
    pub dim: u64,
    pub pos: Pos,
    /// Blocks out from `pos`, along the ground.
    pub radius: u32,
}

impl Spawn {
    /// Whether the block `pos` of dimension `dim` lies in the protected circle.
    pub fn protects(&self, dim: u64, pos: Pos) -> bool {
        let (dx, dz) = (i64::from(pos.x - self.pos.x), i64::from(pos.z - self.pos.z));
        let radius = i64::from(self.radius);
        dim == self.dim && dx * dx + dz * dz <= radius * radius
    }
}

pub fn register(r: &mut Registry) {
    r.command("setspawn", "[radius:int(0,256)]", set)
        .permission(permission::GAME_MASTER);
    r.command("spawn", "", go).permission(permission::EVERYONE);
    r.on_event(&PLAYER_RESPAWNED, |batch| {
        let world = batch.world();
        for respawned in batch.iter() {
            let has_bed = world
                .component(respawned.player, &BED_SPAWN)
                .is_some_and(|bed| bed.set != 0);
            if !has_bed {
                send(&world, respawned.player);
            }
        }
    });
}

/// The spawn, once one was set.
pub fn get() -> Option<Spawn> {
    SPAWN.get(&())
}

/// A player's first join: off to the spawn.
pub fn welcome(world: &World, player: u64) {
    send(world, player);
}

fn set(cmd: &mut ServerCommand) {
    let (world, player) = (cmd.world(), cmd.player());
    let Some(state) = world.entity(player) else {
        cmd.reply("plotty.only_players", &[]);
        return;
    };
    let radius = cmd
        .int("radius")
        .map_or(settings::get().spawn_radius, |given| given as u32);
    let pos = Pos::from(state.pos.block());
    SPAWN.set(
        &(),
        Spawn {
            dim: dims::key_of(&world),
            pos,
            radius,
        },
    );
    cmd.reply(
        "plotty.spawn.set",
        &[
            Arg::Int(pos.x.into()),
            Arg::Int(pos.y.into()),
            Arg::Int(pos.z.into()),
            Arg::Int(radius.into()),
        ],
    );
}

fn go(cmd: &mut ServerCommand) {
    if get().is_none() {
        cmd.reply("plotty.spawn.none", &[]);
        return;
    }
    send(&cmd.world(), cmd.player());
}

/// Moves `player` to the spawn, where one is set and its dimension is still there.
fn send(world: &World, player: u64) {
    let Some(spawn) = get() else { return };
    let Some(dim) = dims::handle(spawn.dim) else {
        return;
    };
    world.teleport(player, Some(dim), spawn.pos.floor_middle());
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPAWN_AT: Spawn = Spawn {
        dim: 7,
        pos: Pos::new(100, 64, -50),
        radius: 10,
    };

    #[test]
    fn the_circle_reaches_its_radius_at_any_height() {
        assert!(
            SPAWN_AT.protects(7, Pos::new(110, 0, -50)),
            "on the edge, far below"
        );
        assert!(
            SPAWN_AT.protects(7, Pos::new(107, 200, -43)),
            "inside, high up"
        );
    }

    #[test]
    fn outside_the_circle_or_dimension_is_not_protected() {
        assert!(
            !SPAWN_AT.protects(7, Pos::new(108, 64, -42)),
            "a corner of the square, outside the circle"
        );
        assert!(
            !SPAWN_AT.protects(8, Pos::new(100, 64, -50)),
            "another dimension"
        );
    }
}
