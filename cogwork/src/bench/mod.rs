//! The assembly bench: makes the recipe of its pattern again and again. The pattern is ghosts,
//! set by clicking its cells with an item; tubes then fill each cell with a stack of its ghost,
//! and engine power turns one item of every cell into the result.

mod cells;
mod craft;
mod ghost;

use base_api::CRAFTING;
pub use ghost::insert;
use weft_sdk::*;

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
/// Side of a slot in GUI pixels, and the player's inventory width.
const SLOT: f32 = 18.0;
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
    r.packet(&ghost::SET_GHOST, ghost::set);
    // On a block without an item, so no item has it.
    r.tag(NOTHING, &["gantry_frame"]);
    r.machine(&ASSEMBLY_BENCH, &BENCH_MENU)
        .block(block)
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

fn block(b: BlockBuilder<'_>) -> BlockBuilder<'_> {
    b.facing(Facings::HORIZONTAL, Toward::Player)
        .prop("working", ["false", "true"])
        .states(|p, s| {
            let mut faces = ["assembly_bench_side"; 6];
            faces[3] = "assembly_bench_top";
            faces[p.facing().face() as usize] = if p.is("working", "true") {
                "assembly_bench_front_on"
            } else {
                "assembly_bench_front"
            };
            s.faces(faces)
        })
        .hardness(3.5)
        .mineable("pickaxe")
        .sound_group("base:block.stone")
        .tab(&crate::TAB)
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
            crate::power_bar(ui);
            ui.text_bind_styled(S::col().no_wrap(), |c, out| {
                let b = c.menu_component(&BENCH).unwrap_or_default();
                let name = status::NAMES.get(b.status as usize).unwrap_or(&"no_recipe");
                out.set(&format!("cogwork.bench.{name}"), &[]);
            });
        });
        ui.player_inventory(true);
    });
}

/// One cell of the pattern, three parts on top of each other: the slot with the real item, a
/// button over it that sets the ghost, and the ghost, drawn smaller. A click reaches the button
/// through the ghost. Item icons in text are drawn after everything, so a cell holding items
/// shows no ghost, which would cover their count.
fn cell(ui: &mut Ui, i: u32) {
    ui.row(S::row().gap(-SLOT), |ui| {
        ui.group_slot("grid", i);
        let size = S::default().class("cogwork:ghost_cell").size(SLOT, SLOT);
        ui.button("", size, move |c| ghost::click(c, i));
        // Text pixels are half a GUI pixel: 1.5 draws the ghost 12 wide, 3 in from the edges.
        ui.column(S::col().size(SLOT, SLOT).pad(3.0), |ui| {
            ui.text_bind_styled(S::col().scale(1.5), move |c, out| {
                let b = c.menu_component(&BENCH).unwrap_or_default();
                let item = b.pattern[i as usize];
                let shown = item != 0 && b.filled & (1 << i) == 0;
                let ghost = if shown {
                    Arg::Item(item)
                } else {
                    Arg::Text("")
                };
                out.set("cogwork.bench.ghost", &[ghost]);
            });
        });
    });
}
