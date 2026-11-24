//! What players do, counted toward their quests: blocks placed and broken, mobs killed, items
//! used and held, and what a quest's own mod reports.

use std::collections::HashMap;

use almanac_api::{ADVANCE, Advance, Kind, Task, book, key_of};
use weft_sdk::*;

use crate::progress::{self, Change};
use crate::targets::{TaskAt, targets};

/// Ticks between two looks at what a player holds.
const HELD_EVERY: u64 = 20;

pub fn register(r: &mut Registry) {
    r.on_event(&actions::PLACE_BLOCK, |batch| {
        let world = batch.world();
        for placed in batch.iter() {
            counted(&world, placed.player, Kind::Place, block_of(placed.state));
        }
    });
    r.on_event(&actions::BREAK_BLOCK, |batch| {
        let world = batch.world();
        for broken in batch.iter() {
            counted(&world, broken.player, Kind::Dig, block_of(broken.state));
        }
    });
    r.on_event(&actions::DIE, |batch| {
        let world = batch.world();
        for death in batch.iter() {
            counted(&world, death.source, Kind::Kill, death.kind);
        }
    });
    r.on_event(&actions::USE_ITEM, |batch| {
        let world = batch.world();
        for used in batch.iter() {
            counted(&world, used.entity, Kind::Use, used.stack.item);
        }
    });
    r.on_event(&ADVANCE, |batch| {
        let world = batch.world();
        for advance in batch.iter() {
            advanced(&world, advance);
        }
    });
    r.world_tick(held);
}

/// One more for every task of `player` that counts `kind` happening to `thing`.
fn counted(world: &World, player: u64, kind: Kind, thing: u32) {
    for &task in targets().counting(kind, thing) {
        progress::count(world, player, task, Change::By(1));
    }
}

/// Counts what a quest's mod reports toward the task it names.
fn advanced(world: &World, advance: &Advance) {
    let book = book();
    let Some(quest) = book.quest(advance.quest) else {
        return;
    };
    let named =
        |task: &Task| task.kind() == Some(Kind::Count) && key_of(&task.name) == advance.task;
    if let Some(task) = book.quests[quest].tasks.iter().position(named) {
        let at = TaskAt { quest, task };
        progress::count(world, advance.player, at, Change::By(advance.amount));
    }
}

/// Counts what each player holds toward their `Kind::Have` tasks, every `HELD_EVERY` ticks of
/// the player's life.
fn held(world: &World) {
    let tasks = &targets().held;
    if tasks.is_empty() {
        return;
    }
    for player in world.players() {
        let Some(state) = world.entity(player) else {
            continue;
        };
        if !state.age.is_multiple_of(HELD_EVERY) {
            continue;
        }
        let items = items_of(world, player);
        for task in tasks {
            let count = task
                .items
                .iter()
                .map(|item| items.get(item).copied().unwrap_or(0))
                .sum();
            if count > 0 {
                progress::count(world, player, task.at, Change::To(count));
            }
        }
    }
}

/// How many of each item `player` has in their inventory.
fn items_of(world: &World, player: u64) -> HashMap<u32, u32> {
    let inventory = world.inventory(player);
    let mut items = HashMap::new();
    for slot in 0..inventory.len() {
        let stack = inventory.get(slot);
        if !stack.is_empty() {
            *items.entry(stack.item).or_insert(0) += stack.count;
        }
    }
    items
}
