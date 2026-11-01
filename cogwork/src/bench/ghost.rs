//! Setting the pattern: clicking a cell of the bench's screen sends what the player holds, and
//! the cell shows it as a ghost; nothing leaves the player's hands. And filling it: items given
//! to the bench go into the cells that ask for them.

use weft_sdk::*;

use super::{BENCH, cells};

value! {
    /// The ghost of `cell` of the bench `owner` is to be `item` (0 clears it).
    pub struct SetGhost {
        pub owner: u64,
        pub cell: u32,
        pub item: u32,
    }
}
pub static SET_GHOST: Channel<SetGhost> = Channel::named("set_ghost");

/// How far from a bench a player may set its pattern, in blocks.
const REACH: f64 = 8.0;

/// Client side of a click on cell `cell`: sends what is on the cursor.
pub fn click(c: &ClientView, cell: u32) {
    let Some(owner) = c.menu_owner() else {
        return;
    };
    let client = c.client();
    let item = client.cursor_stack().item;
    client.send(&SET_GHOST, &SetGhost { owner, cell, item });
}

/// Server side: sets the ghost when the player is at the bench.
pub fn set(p: &Packet, g: SetGhost) {
    let w = p.world();
    let (Some(at), Some(me)) = (w.entity(g.owner), w.entity(p.player())) else {
        return;
    };
    let near = (0..3).map(|i| (at.pos[i] - me.pos[i]).powi(2)).sum::<f64>() <= REACH * REACH;
    let known = g.item == 0 || !item_name(g.item).is_empty();
    if !near || !known || g.cell >= 9 {
        return;
    }
    if let Some(b) = w.component_mut(g.owner, &BENCH) {
        b.pattern[g.cell as usize] = g.item;
    }
}

/// Puts what of `stack` the pattern of the bench `id` still waits for into its cells, one item
/// a cell; returns the rest. `None` when `id` is no bench.
pub fn insert(w: &World, id: u64, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    let pattern = w.component(id, &BENCH)?.pattern;
    let grid = w.inventory(id).group("grid");
    let mut cells: Vec<ItemStack> = (0..grid.len()).map(|i| grid.get(i)).collect();
    let mut left = stack;
    while let Some(i) = cells::free_cell(&pattern, &cells, left.item).filter(|_| !left.is_empty()) {
        cells[i] = ItemStack { count: 1, ..left };
        if !simulate {
            grid.set(i as u32, cells[i]);
        }
        left = left.shrink(1);
    }
    Some(left)
}
