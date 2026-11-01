//! Cogs, the parts every machine here is built from, and the tuning wrench that sets machines.

use weft_sdk::*;

pub static WOODEN_COG: Item = Item::named("wooden_cog");
pub static STONE_COG: Item = Item::named("stone_cog");
pub static IRON_COG: Item = Item::named("iron_cog");
pub static GOLD_COG: Item = Item::named("gold_cog");
pub static DIAMOND_COG: Item = Item::named("diamond_cog");
pub static TUNING_WRENCH: Item = Item::named("tuning_wrench");

pub fn register(r: &mut Registry) {
    for (cog, material) in [
        (&WOODEN_COG, "wood"),
        (&STONE_COG, "stone"),
        (&IRON_COG, "iron"),
        (&GOLD_COG, "gold"),
        (&DIAMOND_COG, "diamond"),
    ] {
        r.item(cog)
            .tag("cogs")
            .tag(&format!("cogs/{material}"))
            .tab(&crate::TAB);
    }
    r.item(&TUNING_WRENCH)
        .max_stack(1)
        .handheld()
        .tag("wrenches")
        .tab(&crate::TAB);
}

/// Whether `player` holds a wrench, this mod's or any other tagged `wrenches`.
pub fn holds_wrench(w: &World, player: u64) -> bool {
    static WRENCHES: Tag = Tag::named("wrenches");
    let held = w.inventory(player).get(w.selected_slot(player));
    !held.is_empty() && WRENCHES.on_item(held.item)
}
