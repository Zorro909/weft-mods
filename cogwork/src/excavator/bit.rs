//! Drill bits: items a quarry digs with, from its one-slot `bit` group. A bit breaks what a
//! pickaxe of its tier breaks, wears one use per block mined, and shows on the end of the rig's
//! shaft (`rig`), a model of its own texture.

use weft_sdk::{
    Component, EntityKind, Item, ItemStack, Registry, StackData, WithComponents, World, abi,
    tool_tier,
};

use crate::lib::BlockState;

/// A kind of drill bit: its item, the pickaxe tier it breaks like, the blocks it lasts and the
/// look of it on the rig.
pub struct Bit {
    pub item: Item,
    pub tier: u32,
    pub lasts: u32,
    pub look: EntityKind,
}

pub static STONE_BIT: Bit = Bit {
    item: Item::named("stone_bit"),
    tier: tool_tier::STONE,
    lasts: 400,
    look: EntityKind::named("stone_drill_bit"),
};
pub static IRON_BIT: Bit = Bit {
    item: Item::named("iron_bit"),
    tier: tool_tier::IRON,
    lasts: 2_000,
    look: EntityKind::named("iron_drill_bit"),
};
pub static STEEL_BIT: Bit = Bit {
    item: Item::named("steel_bit"),
    tier: tool_tier::DIAMOND,
    lasts: 8_000,
    look: EntityKind::named("steel_drill_bit"),
};
/// A steel bit tipped with diamond: it breaks what a steel bit does, five times as long.
pub static DIAMOND_BIT: Bit = Bit {
    item: Item::named("diamond_bit"),
    tier: tool_tier::DIAMOND,
    lasts: 40_000,
    look: EntityKind::named("diamond_drill_bit"),
};
/// Every bit, for registering and for finding one by its item.
pub static BITS: [&Bit; 4] = [&STONE_BIT, &IRON_BIT, &STEEL_BIT, &DIAMOND_BIT];

/// The slot group a quarry keeps its bit in.
pub const GROUP: &str = "bit";

/// The tier a block needs to drop anything: the engine's `harvest_level`, none for most.
static HARVEST_LEVEL: Component<u32> = Component::named(abi::comp::HARVEST_LEVEL);

pub fn register(r: &mut Registry) {
    for bit in BITS {
        r.item(&bit.item)
            .durability(bit.lasts)
            .tag("cogwork:drill_bits")
            .tab(&crate::TAB);
    }
}

/// Whether a drill of `tier` breaks `block_state`: what needs a better pickaxe stays standing.
pub fn breaks(tier: u32, block_state: BlockState) -> bool {
    needed(block_state) <= tier
}

/// The tier a drill needs to break `block_state`.
pub fn needed(block_state: BlockState) -> u32 {
    needs(HARVEST_LEVEL.on_block(block_state.id()))
}

/// The tier a block of `harvest_level` needs, the lowest for one that has none.
fn needs(harvest_level: Option<u32>) -> u32 {
    harvest_level.unwrap_or(tool_tier::WOOD)
}

/// The bit `stack` is, if it is one.
pub fn of(stack: ItemStack) -> Option<&'static Bit> {
    BITS.into_iter()
        .find(|bit| !stack.is_empty() && stack.item == bit.item.id())
}

/// The bit fitted in the quarry `id`; none in an empty slot or one without a bit group.
pub fn fitted(world: &World, id: u64) -> Option<&'static Bit> {
    let group = world.inventory(id).find_group(GROUP)?;
    of(group.get(0))
}

/// Wears the bit fitted in the quarry `id` by one block; a worn-out bit is gone.
pub fn wear(world: &World, id: u64) {
    let Some(group) = world.inventory(id).find_group(GROUP) else {
        return;
    };
    let bit = group.get(0);
    if !bit.is_empty() {
        group.set(0, bit.add_wear(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bit_breaks_what_its_pickaxe_does() {
        let gold = needs(Some(tool_tier::IRON));
        assert!(gold > STONE_BIT.tier, "gold stands against a stone bit");
        assert!(gold <= IRON_BIT.tier, "an iron bit takes it");
        let iron = needs(Some(tool_tier::STONE));
        assert!(iron <= STONE_BIT.tier, "a stone bit takes iron ore");
        let diamond = needs(Some(tool_tier::IRON));
        assert!(
            diamond <= STEEL_BIT.tier,
            "a steel bit takes what iron takes"
        );
        assert!(STEEL_BIT.tier > IRON_BIT.tier, "and more");
        assert_eq!(
            DIAMOND_BIT.tier, STEEL_BIT.tier,
            "diamond breaks what steel does"
        );
        assert_eq!(DIAMOND_BIT.lasts, 5 * STEEL_BIT.lasts, "five times as long");
        assert_eq!(needs(None), tool_tier::WOOD, "dirt needs nothing");
    }

    #[test]
    fn every_bit_has_its_names_and_pictures() {
        for bit in BITS {
            let name = bit.item.path();
            assert!(crate::translated(&format!("item.cogwork.{name}")), "{name}");
            let icon = format!("textures/item/{name}.png");
            assert!(crate::has_asset(&icon), "no {icon}");
            let look = format!("textures/entity/{}.png", bit.look.path());
            assert!(crate::has_asset(&look), "no {look}");
        }
    }
}
