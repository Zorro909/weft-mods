//! A line above the hotbar when a player walks into another part of the world: a city's street,
//! its market, a plot and its owner, the spawn or the wilderness. Players are looked at every
//! half second; what each last saw is kept here, by entity id, which starts over in every world.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use weft_sdk::{Arg, Output, Registry, World};
use weftkit::Pos;

use super::{CellKind, cell_at};
use crate::admin::spawn;
use crate::tables::{CITIES, PLOTS};
use crate::{dims, players};

/// Ticks between looks.
const EVERY: u64 = 10;

/// What each player was last told, as the text of its key and arguments.
static LAST: Mutex<Option<HashMap<u64, String>>> = Mutex::new(None);

pub fn register(r: &mut Registry) {
    r.world_tick(|world| {
        if world.game_time() % EVERY != 0 {
            return;
        }
        for player in world.players_iter() {
            look(world, player);
        }
    });
}

/// Tells `player` where they are, if that changed.
fn look(world: &World, player: u64) {
    let Some(state) = world.entity(player) else {
        return;
    };
    let (key, args) = place_of(world, Pos::from(state.pos.block()));
    let said = format!("{key}{args:?}");
    let mut last = lock();
    let last = last.get_or_insert_with(HashMap::new);
    if last.get(&player) == Some(&said) {
        return;
    }
    last.insert(player, said);
    let args: Vec<Arg> = args.iter().map(|text| Arg::Text(text)).collect();
    world.hint(key, &args).to(player);
}

/// The lang key and arguments saying what the block `pos` belongs to.
fn place_of(world: &World, pos: Pos) -> (&'static str, Vec<String>) {
    let dim = dims::key_of(world);
    let Some(cell) = cell_at(dim, pos) else {
        let at_spawn = spawn::get().is_some_and(|spawn| spawn.protects(dim, pos));
        return (
            if at_spawn {
                "plotty.notice.spawn"
            } else {
                "plotty.notice.wilderness"
            },
            Vec::new(),
        );
    };
    let city = CITIES
        .get(&cell.city)
        .map(|city| city.name)
        .unwrap_or_default();
    let plot = cell
        .plot
        .and_then(|id| PLOTS.get(&id).map(|plot| (id, plot)));
    match (cell.kind, plot) {
        (CellKind::Market, _) => ("plotty.notice.market", vec![city]),
        (_, Some((id, plot))) => match plot.owner {
            Some(owner) => (
                "plotty.notice.plot",
                vec![city, id.to_string(), players::name_of(&owner)],
            ),
            None => ("plotty.notice.free_plot", vec![city, id.to_string()]),
        },
        _ => ("plotty.notice.city", vec![city]),
    }
}

fn lock() -> MutexGuard<'static, Option<HashMap<u64, String>>> {
    LAST.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
