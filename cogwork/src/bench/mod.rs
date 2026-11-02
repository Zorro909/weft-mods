//! The assembly bench: makes the recipe of its pattern again and again. The pattern is ghosts,
//! set by clicking its cells with an item; tubes then fill each cell with a stack of its ghost,
//! and engine power turns one item of every cell into the result.

mod cells;
mod craft;
mod fill;

use base_api::CRAFTING;
pub use fill::insert;
use weft_sdk::*;

use crate::ghost::{self, SLOT};

pub static ASSEMBLY_BENCH: Block = Block::named("assembly_bench");
static BENCH_MENU: Menu = Menu::named("assembly_bench");

value! {
    /// The pattern's ghost items (0 for an empty cell), energy put into the craft under way,
    /// what the screen says, and which cells hold items (a bit each) for the screen. New fields
    /// go last: a saved bench loads with them 0.
    #[derive(Default)]
    pub struct Bench {
        pub pattern: [u32; 9],
        pub spent: i64,
        pub status: u32,
        pub filled: u32,
    }
}
static BENCH: Component<Bench> = Component::named("cogwork:bench");

/// Energy one craft takes.
const CRAFT_COST: i64 = 240;
/// A tag no item has: slots that take it are for showing, never for putting things in.
const NOTHING: &str = "cogwork:nothing";
/// The player's inventory width in GUI pixels.
const INVENTORY_WIDTH: f32 = 9.0 * SLOT;

/// What the screen says, by `cogwork.bench.<name>`.
mod status {
    pub const NO_RECIPE: u32 = 0;
    pub const MISSING: u32 = 1;
    pub const FULL: u32 = 2;
    pub const NO_POWER: u32 = 3;
    pub const WORKING: u32 = 4;
    pub const NAMES: [&str; 5] = ["no_recipe", "missing", "full", "no_power", "working"];
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &BENCH,
        Bench::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&BENCH, craft::tick);
    // A tag has to name something or the game stops at load: the gantry frame has no item, so
    // the tag is on no item.
    r.tag(NOTHING, &["gantry_frame"]);
    r.machine(&ASSEMBLY_BENCH, &BENCH_MENU)
        .block(|b| crate::machine::block(b, "assembly_bench").hardness(3.5))
        // Filled by tubes only, through `insert`; taken from only by crafting.
        .group("grid", Slots::grid(3, 3).input().accepts(NOTHING))
        .group("output", Slots::row(3).output())
        .recipes(&CRAFTING)
        .attach(&BENCH)
        .attach_with(
            &ENERGY,
            Energy {
                stored: 0,
                capacity: 4_000,
                max_in: 200,
                max_out: 0,
            },
        )
        .screen(screen);
}

fn screen(ui: &mut Ui) {
    ui.panel("panel", S::col().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(8.0), |ui| {
            ui.column(S::col(), |ui| {
                for row in 0..3 {
                    ui.row(S::row(), |ui| {
                        (0..3).for_each(|col| cell(ui, row * 3 + col))
                    });
                }
            });
            ui.recipe_arrow_bind(|c| {
                let b = c.menu_component(&BENCH).unwrap_or_default();
                b.spent as f32 / CRAFT_COST as f32
            });
            ui.group("output");
        });
        // As wide as the inventory whatever it says, so the cells don't move under the cursor.
        ui.column(S::col().gap(3.0).w(INVENTORY_WIDTH), |ui| {
            crate::machine::power_bar(ui);
            ui.text_bind_styled(S::col().no_wrap(), |c, out| {
                let b = c.menu_component(&BENCH).unwrap_or_default();
                let name = status::NAMES.get(b.status as usize).unwrap_or(&"no_recipe");
                out.set(&format!("cogwork.bench.{name}"), &[]);
            });
        });
        ui.player_inventory(true);
    });
}

/// One cell of the pattern: the slot with the real items under its ghost. Item icons in text
/// are drawn after everything, so a cell holding items shows no ghost, which would cover their
/// count.
fn cell(ui: &mut Ui, i: u32) {
    let under = |ui: &mut Ui| ui.group_slot("grid", i);
    ghost::cell(ui, under, i, move |c| {
        let b = c.menu_component(&BENCH).unwrap_or_default();
        let filled = b.filled & (1 << i) != 0;
        if filled { 0 } else { b.pattern[i as usize] }
    });
}

/// Sets the ghost of `cell` when `owner` is a bench.
pub fn set_ghost(w: &World, owner: u64, cell: u32, item: u32) {
    if let Some(b) = w.component_mut(owner, &BENCH)
        && let Some(ghost) = b.pattern.get_mut(cell as usize)
    {
        *ghost = item;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for name in status::NAMES {
            let key = format!("cogwork.bench.{name}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
