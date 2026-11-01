//! Mining one block: the drill travels to it, energy is paid by how hard it is, and what it drops
//! goes into the excavator's output, as if mined with a diamond pickaxe.

use weft_sdk::*;

use super::power::{pay, travel};
use super::{Dig, EXCAVATOR, GANTRY_FRAME, status};

/// Energy per block: a base and as much again per point of hardness (stone 1.5: 1000).
const BASE_COST: f32 = 400.0;
const COST_PER_HARDNESS: f32 = 400.0;
/// Blocks per tick the drill moves.
const DRILL_SPEED: f32 = 0.25;

static TOOL: Item = Item::named("base:diamond_pickaxe");

pub enum Mined {
    Done,
    /// Nothing to mine there after all, or someone wouldn't let it.
    Skipped,
    /// Not yet: the status says why.
    Waiting(u32),
}

/// A block's loot table, by the block's name.
struct TableOf(&'static str);

impl LootSource for TableOf {
    fn table_name(&self) -> &str {
        self.0
    }
}

/// Whether a block can simply be built into: air, plants, liquids.
pub fn opens_into(state: u32) -> bool {
    state == 0
        || state_flags(state) & (abi::block_flags::REPLACEABLE | abi::block_flags::LIQUID) != 0
}

/// Whether the drill stops for a block: not air or liquid, breakable, and not its own frame.
pub fn worth_mining(state: u32) -> bool {
    state != 0
        && state_flags(state) & abi::block_flags::LIQUID == 0
        && HARDNESS.on_block(state).is_none_or(|h| h >= 0.0)
        && !GANTRY_FRAME.is(state)
        && !EXCAVATOR.is(state)
}

/// Moves the drill toward `target` as far as this tick's energy pays for; true once it is there.
pub fn drill_to(w: &World, id: u64, d: &mut Dig, target: [i32; 3]) -> bool {
    let goal = [
        target[0] as f32 + 0.5,
        target[1] as f32,
        target[2] as f32 + 0.5,
    ];
    let gap: [f32; 3] = std::array::from_fn(|i| goal[i] - d.head[i]);
    let length = gap.iter().map(|v| v * v).sum::<f32>().sqrt();
    let step = travel(w, id, length.min(DRILL_SPEED));
    if step >= length {
        d.head = goal;
        return true;
    }
    d.status = if step > 0.0 {
        status::MINING
    } else {
        status::NO_POWER
    };
    for i in 0..3 {
        d.head[i] += gap[i] / length * step;
    }
    false
}

/// Pays for and mines the block `state` at `c` into the excavator `id`'s output.
pub fn mine_block(w: &World, id: u64, d: &mut Dig, c: [i32; 3], state: u32) -> Mined {
    if !worth_mining(state) {
        return Mined::Skipped;
    }
    let cost = (BASE_COST + COST_PER_HARDNESS * HARDNESS.on_block(state).unwrap_or(1.0)) as i64;
    d.status = status::MINING;
    if !pay(w, id, d, cost) {
        return Mined::Waiting(d.status);
    }
    let tool = ItemStack::new(TOOL.id(), 1);
    let name = block_name(state).split('[').next().unwrap_or("");
    let drops = w.roll_loot(
        &TableOf(name),
        &LootCtx {
            tool,
            mined: state,
            ..LootCtx::default()
        },
    );
    let output = || w.inventory(id).group("output");
    if drops
        .iter()
        .any(|s| !crate::store::insert_into(output(), *s, true).is_empty())
    {
        // Paid for already: it stays paid until there is room.
        return Mined::Waiting(status::FULL);
    }
    d.spent = 0;
    if !w.break_block(c, tool, 0, false) {
        return Mined::Skipped;
    }
    for s in drops {
        let left = crate::store::insert_into(output(), s, false);
        if !left.is_empty() {
            w.spawn_item(c.map(|v| v as f64 + 0.5), left);
        }
    }
    Mined::Done
}
