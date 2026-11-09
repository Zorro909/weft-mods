//! Cogs, the parts every machine here is built from, and the tuning wrench that sets machines.

use weft_sdk::*;

use crate::engines::{EMBER_ENGINE, SPRING_ENGINE};
use crate::lib::{Pos, TypedWorld};
use crate::tube::DIRECTING_TUBE;

pub static WOODEN_COG: Item = Item::named("wooden_cog");
pub static STONE_COG: Item = Item::named("stone_cog");
pub static IRON_COG: Item = Item::named("iron_cog");
pub static GOLD_COG: Item = Item::named("gold_cog");
pub static DIAMOND_COG: Item = Item::named("diamond_cog");
pub static TUNING_WRENCH: Item = Item::named("tuning_wrench");

pub fn register(r: &mut Registry) {
    cog(r, &WOODEN_COG, "wood");
    cog(r, &STONE_COG, "stone");
    cog(r, &IRON_COG, "iron");
    cog(r, &GOLD_COG, "gold");
    cog(r, &DIAMOND_COG, "diamond");
    r.item(&TUNING_WRENCH)
        .max_stack(1)
        .handheld()
        .tag("wrenches")
        .tab(&crate::TAB)
        .on_use(turn_aimed);
}

/// A cog, tagged `cogs` and `cogs/<material>` so recipes of any mod can ask for one or the
/// other.
fn cog(r: &mut Registry, cog: &'static Item, material: &str) {
    r.item(cog)
        .tag("cogs")
        .tag(&format!("cogs/{material}"))
        .tab(&crate::TAB);
}

/// The tuning wrench used on a block that leaves the click to it, as every block does for a
/// sneaking player: this is how it turns an ember engine, which opens its screen otherwise.
fn turn_aimed(u: &mut ItemUse) -> bool {
    let Some((pos, _)) = u.target_block() else {
        return false;
    };
    let (w, pos) = (u.world(), Pos::from(pos));
    let block = w.block_typed(pos);
    if block.is(&SPRING_ENGINE) || block.is(&EMBER_ENGINE) {
        crate::engines::turn(&w, pos);
    } else if block.is(&DIRECTING_TUBE) {
        crate::tube::turn_directing(&w, pos);
    } else {
        return false;
    }
    true
}

/// Whether `player` holds a wrench, this mod's or any other tagged `wrenches`.
pub fn holds_wrench(w: &World, player: u64) -> bool {
    static WRENCHES: Tag = Tag::named("wrenches");
    let held = w.inventory(player).get(w.selected_slot(player));
    !held.is_empty() && WRENCHES.on_item(held.item)
}
