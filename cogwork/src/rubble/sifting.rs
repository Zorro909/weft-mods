//! What sifting one Rubble gives: mostly sand and gravel, and rarely something worth more than
//! stone. Each find has its own chance, rolled on its own, so one Rubble may give several or
//! none. The sieve sifts by this table, and so will the shaker screen.

use weft_sdk::{Item, ItemStack, World};

use super::{Grade, Ore, ore_rubble};
use crate::chance;

/// One thing sifting may turn up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Find {
    /// An item of any mod, by its full name.
    Item(&'static str),
    /// Poor Ore Rubble of a metal.
    OreRubble(&'static str),
}

/// What one Rubble may give, and the chance of each.
pub const RUBBLE: [(Find, f64); 9] = [
    (Find::Item("base:sand"), 0.60),
    (Find::Item("base:gravel"), 0.25),
    (Find::Item("base:flint"), 0.08),
    (Find::Item("base:clay_ball"), 0.05),
    (Find::Item("base:coal"), 0.02),
    (Find::OreRubble("iron"), 0.015),
    (Find::Item("base:redstone"), 0.005),
    (Find::OreRubble("gold"), 0.004),
    (Find::Item("base:diamond"), 0.0005),
];

/// What `table` turns up, `roll` giving a number from 0 to 1 for each find in turn.
pub fn sift(table: &[(Find, f64)], mut roll: impl FnMut() -> f64) -> Vec<Find> {
    table
        .iter()
        .filter(|&&(_, odds)| chance::happens(odds, roll()))
        .map(|&(find, _)| find)
        .collect()
}

/// The stacks one Rubble sifts into in `world`.
pub fn sift_rubble(world: &World) -> Vec<ItemStack> {
    let finds = sift(&RUBBLE, || chance::roll(world));
    finds.into_iter().filter_map(stack_of).collect()
}

/// One of `find`, none when no loaded mod has it.
fn stack_of(find: Find) -> Option<ItemStack> {
    match find {
        Find::Item(name) => Some(Item::named(name).stack(1)),
        Find::OreRubble(metal) => Some(ore_rubble(Ore::of(metal)?, Grade::Poor, 1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_find_is_rolled_on_its_own() {
        let mut rolls = [0.5, 0.5, 0.01, 0.9, 0.9, 0.9, 0.9, 0.9, 0.9].into_iter();
        let finds = sift(&RUBBLE, || rolls.next().unwrap_or(1.0));
        let expected = [Find::Item("base:sand"), Find::Item("base:flint")];
        assert_eq!(finds, expected, "sand and flint, not gravel");
    }

    #[test]
    fn a_low_roll_finds_everything_and_a_high_one_nothing() {
        assert_eq!(sift(&RUBBLE, || 0.0).len(), RUBBLE.len(), "everything");
        assert!(sift(&RUBBLE, || 0.9999).is_empty(), "nothing");
    }

    #[test]
    fn something_worth_more_than_stone_is_rare() {
        let worth_more = ["base:coal", "base:redstone", "base:diamond"];
        let rare: f64 = RUBBLE
            .iter()
            .filter(|(find, _)| match find {
                Find::Item(name) => worth_more.contains(name),
                Find::OreRubble(_) => true,
            })
            .map(|(_, odds)| odds)
            .sum();
        assert!(rare < 0.05, "under one in twenty: {rare}");
    }

    #[test]
    fn its_metals_are_ones_a_stack_keeps() {
        for (find, _) in RUBBLE {
            if let Find::OreRubble(metal) = find {
                assert!(Ore::of(metal).is_some_and(|ore| ore.named > 0), "{metal}");
            }
        }
    }
}
