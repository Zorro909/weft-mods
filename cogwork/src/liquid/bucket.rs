//! Buckets on a holder of liquid: a full one pours its liquid in, an empty one is filled from
//! it. Which bucket holds which liquid is any mod's say: an item with `LIQUID` names the liquid it
//! holds and `REMAINDER` the empty container it leaves.

use std::collections::HashMap;
use std::sync::OnceLock;

use weft_sdk::*;

use super::vessel::{Holders, Vessel};
use super::{BUCKET, LiquidKey, key_named};

/// What a full container holds and leaves, by its item: the liquid and the empty item.
type Contents = HashMap<u32, (LiquidKey, u32)>;
/// The full container, by the empty item and the liquid.
type Fillings = HashMap<(u32, LiquidKey), u32>;

/// Pours the bucket `player` holds into `vessel`, or fills it from there; false when the item
/// is no bucket for this holder, or there is no room or not a bucket's worth.
pub fn use_bucket(holders: &mut Holders, vessel: &Vessel, player: u64) -> bool {
    let world = *holders.world();
    let slot = world.selected_slot(player);
    let held = world.inventory(player).get(slot);
    if held.is_empty() {
        return false;
    }
    let (contents, fillings) = containers();
    let now = vessel.held(holders);
    let next = if let Some(&(liquid, empty)) = contents.get(&held.item) {
        let room = vessel.capacity() - now.amount.min(vessel.capacity());
        if !now.takes(liquid, vessel.capacity()) || room < BUCKET {
            return false;
        }
        vessel.put(holders, liquid, BUCKET);
        empty
    } else {
        let Some(&full) = fillings.get(&(held.item, now.liquid)) else {
            return false;
        };
        if now.amount < BUCKET {
            return false;
        }
        vessel.take(holders, BUCKET);
        full
    };
    hand_over(&world, player, slot, held, next);
    true
}

/// Swaps one of `held` in `slot` of `player` for `next`; a player in creative keeps the bucket.
fn hand_over(world: &World, player: u64, slot: u32, held: ItemStack, next: u32) {
    if world.creative(player) {
        return;
    }
    let inventory = world.inventory(player);
    let next = ItemStack::new(next, 1);
    if held.count == 1 {
        inventory.set(slot, next);
        return;
    }
    inventory.set(slot, held.shrink(1));
    let left = inventory.insert(next);
    if !left.is_empty() {
        world.spawn_item(world.entity(player).map_or(Vec3::ZERO, |e| e.pos), left);
    }
}

/// Every container of every mod, read once: the items don't change while the game runs.
fn containers() -> &'static (Contents, Fillings) {
    static TABLES: OnceLock<(Contents, Fillings)> = OnceLock::new();
    TABLES.get_or_init(|| {
        let (mut contents, mut fillings) = (HashMap::new(), HashMap::new());
        for item in 1..entry_count(abi::entry_kind::ITEM) {
            let Some(liquid) = LIQUID.on_item(item) else {
                continue;
            };
            let liquid = key_named(liquid.as_str());
            let empty = REMAINDER.on_item(item).map_or(0, |empty| empty.0);
            contents.insert(item, (liquid, empty));
            if empty != 0 {
                fillings.insert((empty, liquid), item);
            }
        }
        (contents, fillings)
    })
}
