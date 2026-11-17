//! Breaking a block by machine: what it drops, as if mined with a diamond pickaxe, goes into the
//! machine's output. Shared by the excavator, the leveler and the pump: the leveler and the pump
//! keep the drops as they are, a quarry brings stone and ore up as rubble (`rubble::quarried`).
//! A block that keeps what it holds on its item (a loaded millstone or sieve) keeps it here too.

use weft_sdk::{Component, Item, ItemStack, LootCtx, LootSource, StackData, Value, World};

use crate::lib::{BlockState, Pos};

static TOOL: Item = Item::named("base:diamond_pickaxe");

/// A block's loot table, by the block's name.
struct TableOf(&'static str);

impl LootSource for TableOf {
    fn table_name(&self) -> &str {
        self.0
    }
}

/// What a machine keeps of the drops of what it breaks.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Keep {
    /// The drops as they are.
    Clean,
    /// A quarry's: stone as Rubble, and `multiplier` Ore Rubble of a metal ore.
    Quarry { multiplier: f64 },
}

pub enum Harvest {
    Done,
    /// The output has no room for what it would drop: nothing changed.
    Full,
    /// Someone wouldn't let it break the block.
    Refused,
}

/// Energy to break `block_state`: `base`, and as much again as `per_hardness` per point of
/// hardness.
pub fn cost(block_state: BlockState, base: f32, per_hardness: f32) -> i64 {
    (base + per_hardness * block_state.hardness().unwrap_or(1.0)) as i64
}

/// Breaks `block_state` at `cell` into the `output` group of the machine `id`, keeping of its
/// drops what `keep` says, only when all of that fits there.
pub fn harvest(world: &World, id: u64, cell: Pos, block_state: BlockState, keep: Keep) -> Harvest {
    let output = || world.inventory(id).group("output");
    let tool = ItemStack::new(TOOL.id(), 1);
    // The loot table is the block's, whatever its properties.
    let name = block_state.block_name();
    let context = LootCtx {
        tool,
        mined: block_state.id(),
        ..LootCtx::default()
    };
    let drops = world.roll_loot(&TableOf(name), &context);
    let drops = match keep {
        Keep::Clean => drops,
        Keep::Quarry { multiplier } => {
            crate::rubble::quarried(world, block_state, drops, multiplier)
        }
    };
    let is_own =
        |item| BlockState::placed_by(item).is_some_and(|placed| placed.block_name() == name);
    let drops = carry_onto(drops, is_own, |stack| {
        with_carried(world, world.block_entity(cell), stack)
    });
    if drops
        .iter()
        .any(|stack| !crate::store::insert_into(output(), *stack, true).is_empty())
    {
        return Harvest::Full;
    }
    if !world.break_block(cell, tool, 0, false) {
        return Harvest::Refused;
    }
    for stack in drops {
        let left = crate::store::insert_into(output(), stack, false);
        if !left.is_empty() {
            world.spawn_item(cell.center(), left);
        }
    }
    Harvest::Done
}

/// `drops` with `carry` done to the first stack that `is_own`, the broken block's own item, as
/// the engine puts what a block entity carries on the item it drops.
fn carry_onto(
    mut drops: Vec<ItemStack>,
    is_own: impl Fn(u32) -> bool,
    carry: impl FnOnce(ItemStack) -> ItemStack,
) -> Vec<ItemStack> {
    if let Some(stack) = drops.iter_mut().find(|stack| is_own(stack.item)) {
        *stack = carry(*stack);
    }
    drops
}

/// `stack` with what the block entity `entity` carries. The engine does this only for the drops
/// `break_block` makes, which a machine goes without, and the SDK reads a component only by its
/// handle: so these are cogwork's own, the millstone's and the sieve's.
fn with_carried(world: &World, entity: u64, stack: ItemStack) -> ItemStack {
    let stack = with_component(world, entity, stack, &crate::millstone::MILL);
    with_component(world, entity, stack, &crate::sieve::SIFT)
}

/// `stack` with `entity`'s value of `component`, if it has one.
fn with_component<T: Value>(
    world: &World,
    entity: u64,
    stack: ItemStack,
    component: &Component<T>,
) -> ItemStack {
    match world.component(entity, component) {
        Some(value) => stack.with(component, value),
        None => stack,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MILLSTONE: u32 = 40;
    const SIEVE: u32 = 41;
    const RUBBLE: u32 = 42;
    /// Stands for the stack data of what a block entity carries.
    const CARRIED: u32 = 7;

    fn carried(stack: ItemStack) -> ItemStack {
        ItemStack {
            data: CARRIED,
            ..stack
        }
    }

    #[test]
    fn a_broken_millstone_keeps_what_it_holds() {
        let drops = vec![ItemStack::new(MILLSTONE, 1)];
        let kept = carry_onto(drops, |item| item == MILLSTONE, carried);
        assert_eq!(
            kept[0].data, CARRIED,
            "its rubble, water and sand on its item"
        );
    }

    #[test]
    fn a_broken_sieve_keeps_its_load_on_its_own_item_only() {
        let drops = vec![ItemStack::new(RUBBLE, 2), ItemStack::new(SIEVE, 1)];
        let kept = carry_onto(drops, |item| item == SIEVE, carried);
        assert_eq!(kept[0].data, 0, "not on the rubble beside it");
        assert_eq!(kept[1].data, CARRIED, "on the sieve");
    }

    #[test]
    fn what_a_block_carries_goes_onto_one_stack() {
        let drops = vec![ItemStack::new(MILLSTONE, 1), ItemStack::new(MILLSTONE, 1)];
        let kept = carry_onto(drops, |item| item == MILLSTONE, carried);
        let data: Vec<u32> = kept.iter().map(|stack| stack.data).collect();
        assert_eq!(data, [CARRIED, 0], "never twice");
    }

    #[test]
    fn drops_without_the_block_take_nothing() {
        let drops = vec![ItemStack::new(RUBBLE, 1)];
        let kept = carry_onto(drops, |item| item == MILLSTONE, |_| unreachable!());
        assert_eq!(kept, [ItemStack::new(RUBBLE, 1)], "as they were");
    }
}
