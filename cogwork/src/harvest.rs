//! Breaking a block by machine: what it drops, as if mined with a diamond pickaxe, goes into the
//! machine's output. Shared by the excavator and the leveler.

use weft_sdk::*;

use crate::block_state::BlockState;
use crate::pos::Pos;

static TOOL: Item = Item::named("base:diamond_pickaxe");

/// A block's loot table, by the block's name.
struct TableOf(&'static str);

impl LootSource for TableOf {
    fn table_name(&self) -> &str {
        self.0
    }
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

/// Breaks `block_state` at `c` into the `output` group of the machine `id`, only when
/// everything it drops fits there.
pub fn harvest(w: &World, id: u64, c: Pos, block_state: BlockState) -> Harvest {
    let output = || w.inventory(id).group("output");
    let tool = ItemStack::new(TOOL.id(), 1);
    // The loot table is the block's, whatever its properties.
    let name = block_state.name().split('[').next().unwrap_or("");
    let ctx = LootCtx {
        tool,
        mined: block_state.id(),
        ..LootCtx::default()
    };
    let drops = w.roll_loot(&TableOf(name), &ctx);
    if drops
        .iter()
        .any(|s| !crate::store::insert_into(output(), *s, true).is_empty())
    {
        return Harvest::Full;
    }
    if !w.break_block(c.into(), tool, 0, false) {
        return Harvest::Refused;
    }
    for s in drops {
        let left = crate::store::insert_into(output(), s, false);
        if !left.is_empty() {
            w.spawn_item(c.center().into(), left);
        }
    }
    Harvest::Done
}
