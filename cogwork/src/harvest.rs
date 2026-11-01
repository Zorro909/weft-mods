//! Breaking a block by machine: what it drops, as if mined with a diamond pickaxe, goes into the
//! machine's output. Shared by the excavator and the leveler.

use weft_sdk::*;

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

/// Whether a block can simply be built into: air, plants, liquids.
pub fn opens_into(state: u32) -> bool {
    state == 0
        || state_flags(state) & (abi::block_flags::REPLACEABLE | abi::block_flags::LIQUID) != 0
}

/// Whether a machine can break the block: not air or liquid, and not unbreakable.
pub fn breakable(state: u32) -> bool {
    state != 0
        && state_flags(state) & abi::block_flags::LIQUID == 0
        && HARDNESS.on_block(state).is_none_or(|h| h >= 0.0)
}

/// Energy to break the block: `base`, and as much again as `per_hardness` per point of hardness.
pub fn cost(state: u32, base: f32, per_hardness: f32) -> i64 {
    (base + per_hardness * HARDNESS.on_block(state).unwrap_or(1.0)) as i64
}

/// Breaks the block `state` at `c` into the `output` group of the machine `id`, only when
/// everything it drops fits there.
pub fn harvest(w: &World, id: u64, c: [i32; 3], state: u32) -> Harvest {
    let output = || w.inventory(id).group("output");
    let tool = ItemStack::new(TOOL.id(), 1);
    let name = block_name(state).split('[').next().unwrap_or("");
    let ctx = LootCtx {
        tool,
        mined: state,
        ..LootCtx::default()
    };
    let drops = w.roll_loot(&TableOf(name), &ctx);
    if drops
        .iter()
        .any(|s| !crate::store::insert_into(output(), *s, true).is_empty())
    {
        return Harvest::Full;
    }
    if !w.break_block(c, tool, 0, false) {
        return Harvest::Refused;
    }
    for s in drops {
        let left = crate::store::insert_into(output(), s, false);
        if !left.is_empty() {
            w.spawn_item(c.map(|v| v as f64 + 0.5), left);
        }
    }
    Harvest::Done
}
