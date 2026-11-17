//! What a quarry brings up of a block it mined, made from the block's own drops: stone comes up
//! as Rubble, a metal ore as Poor Ore Rubble, and a gem ore (coal, diamond, redstone) as its
//! drops. Of an ore it brings up as much as its multiplier says, the fraction a chance of one
//! more. Dirt, gravel, sand and everything else come up as they drop.

use weft_sdk::{Item, ItemStack, Tag, World, item_block};

use super::metal::{Vein, vein};
use super::{Grade, Ore, ore_rubble};
use crate::chance;
use crate::lib::BlockState;

static RUBBLE: Item = Item::named("rubble");
static STONE: Tag = Tag::named("stone");
static COBBLESTONE: Tag = Tag::named("cobblestone");

/// What a quarry with `multiplier` brings up of `block_state`, which drops `drops`.
pub fn quarried(
    world: &World,
    block_state: BlockState,
    drops: Vec<ItemStack>,
    multiplier: f64,
) -> Vec<ItemStack> {
    let quarried: Vec<ItemStack> = match vein(block_state) {
        Vein::Metal(metal) => match Ore::of(metal) {
            Some(ore) => {
                let count = chance::count(multiplier, chance::roll(world));
                vec![ore_rubble(ore, Grade::Poor, count)]
            }
            None => drops,
        },
        Vein::Gem => drops
            .into_iter()
            .map(|stack| more(stack, multiplier, chance::roll(world)))
            .collect(),
        Vein::Rock => drops.into_iter().map(as_rubble).collect(),
    };
    quarried
        .into_iter()
        .filter(|stack| !stack.is_empty())
        .collect()
}

/// `stack` with its count times `multiplier`, the fraction a chance of one more by `roll`.
fn more(stack: ItemStack, multiplier: f64, roll: f64) -> ItemStack {
    let count = chance::count(stack.count as f64 * multiplier, roll);
    ItemStack { count, ..stack }
}

/// `stack` as Rubble when it is stone or cobblestone, any mod's; else as it is.
fn as_rubble(stack: ItemStack) -> ItemStack {
    let block = item_block(stack.item);
    let tagged = |tag: &Tag| tag.on_item(stack.item) || block.is_some_and(|b| tag.on_block(b));
    if tagged(&STONE) || tagged(&COBBLESTONE) {
        RUBBLE.stack(stack.count)
    } else {
        stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_gem_ore_brings_up_its_drops_times_the_multiplier() {
        let coal = ItemStack::new(7, 1);
        assert_eq!(more(coal, 2.0, 0.9).count, 2, "the gantry doubles it");
        assert_eq!(
            more(coal, 1.25, 0.1).count,
            2,
            "a quarter chance of one more"
        );
        assert_eq!(more(coal, 1.25, 0.5).count, 1, "missed");
        let redstone = ItemStack::new(8, 4);
        assert_eq!(more(redstone, 1.5, 0.9).count, 6, "each of several");
    }
}
