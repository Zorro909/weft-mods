//! Breaking a block by machine: what it drops, as if mined with a diamond pickaxe, goes into the
//! machine's output. Shared by the excavator, the leveler and the pump: the leveler and the pump
//! keep the drops as they are, a quarry brings stone and ore up as rubble (`rubble::quarried`).

use weft_sdk::{Item, ItemStack, LootCtx, LootSource, World};

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
