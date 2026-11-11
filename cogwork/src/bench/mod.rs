//! The assembly bench: makes the recipe of its pattern again and again. The pattern is ghosts,
//! set by clicking its cells with an item; tubes then fill each cell with a stack of its ghost,
//! and engine power turns one item of every cell into the result.

mod cells;
mod craft;
mod fill;

use base_api::CRAFTING;
pub use fill::insert;
use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::*;

use crate::ghost::{self, ItemKey, SLOT};

pub static ASSEMBLY_BENCH: Block = Block::named("assembly_bench");
static BENCH_MENU: Menu = Menu::named("assembly_bench");

value! {
    /// The pattern's ghost items (`ghost::NO_ITEM` for an empty cell), energy put into the
    /// craft under way, what the screen says, and which cells hold items (a bit each) for the
    /// screen. New fields go last: a saved bench loads with them 0.
    #[derive(Default)]
    pub struct Bench {
        pub pattern: [ItemKey; 9],
        pub spent: i64,
        pub status: u32,
        pub filled: u32,
    }
}

/// Named anew when the pattern became item keys, so a bench saved before starts with an empty
/// pattern rather than reading item numbers as keys.
static BENCH: Component<Bench> = Component::named("cogwork:assembly_bench");

/// Energy one craft takes.
const CRAFT_COST: i64 = 240;
/// A tag no item has: slots that take it are for showing, never for putting things in.
const NOTHING: &str = "cogwork:nothing";
/// The player's inventory width in GUI pixels.
const INVENTORY_WIDTH: f32 = 9.0 * SLOT;

/// What the bench is doing; its screen says `cogwork.bench.<name>`. Kept in
/// `Bench::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
enum Status {
    #[default]
    NoRecipe,
    Missing,
    Full,
    NoPower,
    Working,
}

impl Bench {
    fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }
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
    crate::machine::block(r.block(&ASSEMBLY_BENCH), "assembly_bench")
        .hardness(3.5)
        .entity(|entity| {
            entity
                .attach(&BENCH)
                .attach_with(&ENERGY, crate::machine::energy(4_000, 200))
                // The grid, then the output.
                .inventory(9 + 3)
        })
        .opens(&BENCH_MENU);
    r.menu(&BENCH_MENU)
        // Filled by tubes only, through `insert`; taken from only by crafting.
        .group(
            "grid",
            Slots::grid(3, 3)
                .input()
                .accepts(&format!("#{NOTHING}"))
                .holder(),
        )
        .group("output", Slots::row(3).output().holder())
        .recipes(&CRAFTING)
        .player_inventory()
        .screen(screen);
}

/// The bench's screen:
///
/// ```text
/// +--------------------------------------+
/// | Assembly Bench                       |
/// | [ ][ ][ ]                            |
/// | [ ][ ][ ]  ==>  [ ][ ][ ]            |  pattern, craft progress, output
/// | [ ][ ][ ]                            |
/// | [#####.....]                         |  stored energy
/// | 1200 / 4000 energy                   |
/// | Crafting                             |  status
/// |                                      |
/// | [ ][ ][ ][ ][ ][ ][ ][ ][ ]          |  the player's inventory
/// | [ ][ ][ ][ ][ ][ ][ ][ ][ ]          |
/// | [ ][ ][ ][ ][ ][ ][ ][ ][ ]          |
/// | [ ][ ][ ][ ][ ][ ][ ][ ][ ]          |
/// +--------------------------------------+
/// ```
///
/// A pattern cell shows its ghost item until it holds items.
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(8.0), |ui| {
            ui.column(S::column(), |ui| {
                for row in 0..3 {
                    ui.row(S::row(), |ui| {
                        (0..3).for_each(|col| cell(ui, row * 3 + col))
                    });
                }
            });
            ui.recipe_arrow_bind(|view| {
                let bench = view.menu_component(&BENCH).unwrap_or_default();
                bench.spent as f32 / CRAFT_COST as f32
            });
            ui.group("output");
        });
        // As wide as the inventory whatever it says, so the cells don't move under the cursor.
        ui.column(S::column().gap(3.0).w(INVENTORY_WIDTH), |ui| {
            crate::machine::power_bar(ui);
            ui.text_bind_styled(S::column().no_wrap(), |view, out| {
                let bench = view.menu_component(&BENCH).unwrap_or_default();
                let status: &str = bench.status().into();
                out.set(&format!("cogwork.bench.{status}"), &[]);
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
    ghost::cell(ui, under, i, move |view| {
        let bench = view.menu_component(&BENCH).unwrap_or_default();
        let filled = bench.filled & (1 << i) != 0;
        if filled {
            0
        } else {
            ghost::item_of(bench.pattern[i as usize])
        }
    });
}

/// The items of the bench's pattern, 0 for an empty cell.
fn pattern(bench: &Bench) -> [u32; 9] {
    bench.pattern.map(ghost::item_of)
}

/// Sets the ghost of `cell` to `item` (0 clears it) when `owner` is a bench.
pub fn set_ghost(world: &World, owner: u64, cell: u32, item: u32) {
    let Some(bench) = world.component_mut(owner, &BENCH) else {
        return;
    };
    let Some(ghost) = bench.pattern.get_mut(cell as usize) else {
        return;
    };
    *ghost = ghost::key_of(item);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.bench.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }
}
