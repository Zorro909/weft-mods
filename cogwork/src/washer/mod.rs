//! The washer: a trough with a paddle wheel that washes Poor Ore Rubble into Common, the first
//! step of raising a grade (`rubble::Grade`). Ore Rubble comes in by tube or by hand, water by
//! liquid tube (it is a holder of liquid that takes only water, like the grinding mill), and
//! with engine energy the paddle washes each Poor Ore Rubble in turn. Ore Rubble already Common
//! or better passes through untouched. What it washes leaves by its front, the side that faced
//! the player who placed it, into a tube or a machine there; a timber tube can pull it out too.
//!
//! Its shape is the Blockbench model `assets/models/block/washer.json`, an iron-rimmed wooden
//! trough with a bearing on each side, and `washer_water.json` with water in it; the paddle is
//! `assets/models/washer_paddle.geo.json` (`paddle`).

mod paddle;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Facings, Holder, Holds, ItemStack, Menu, Registry, S, Slots,
    Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::liquid::HELD;
use crate::rubble::{ORE_RUBBLE, ore_of};

pub static WASHER: Block = Block::named("washer");
static WASHER_MENU: Menu = Menu::named("washer");
/// The water in its trough: base's grey water picture, tinted by the biome as water is.
const WATER: &str = "base:water_still@core:water";

/// Millibuckets of water it holds: four washes.
pub const WATER_CAPACITY: u32 = 1_000;
/// Its slots: Ore Rubble to wash, then what it washed.
const INPUT: u32 = 3;
const OUTPUT: u32 = 3;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 150;
const STORES: i64 = 2_000;

value! {
    /// A washer's state: its `paddle` (an entity), the energy `spent` on the Ore Rubble it
    /// washes, which stays in its input until it is washed, and its `status`.
    #[derive(Default)]
    pub struct Wash {
        pub paddle: u64,
        pub spent: i64,
        pub status: u32,
    }
}
static WASH: Component<Wash> = Component::named("cogwork:wash");

/// What the washer is doing or lacks; its screen says `cogwork.washer.<name>`. Kept in
/// `Wash::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoPower,
    Washing,
    NoWater,
    /// What it would wash, or pass through, has no room in its output.
    Full,
}

impl Wash {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &WASH,
        Wash::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&WASH, work::tick);
    r.block(&WASHER)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("washer")
        .prop("water", ["false", "true"])
        .states(|props, state| {
            if props.is("water", "true") {
                state
                    .model_file("washer_water")
                    .model_texture("water", WATER)
            } else {
                state
            }
        })
        .hardness(2.5)
        .mined_by("axe", tool_tier::HAND)
        .sound_group("base:block.wood")
        .entity(|entity| {
            entity
                .attach(&WASH)
                .attach(&HELD)
                .attach(&crate::owned::LOOPING)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(INPUT + OUTPUT)
        })
        .opens(&WASHER_MENU)
        .tab(&crate::TAB);
    r.menu(&WASHER_MENU)
        .group(
            "input",
            Slots::row(INPUT).accepts(ORE_RUBBLE.name()).holder(),
        )
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    paddle::register(r);
}

/// Puts `stack` from a tube into the washer `at`: Ore Rubble into its input, nothing else. None
/// for what is no washer.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &WASH)?;
    if ore_of(&stack).is_none() {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("input")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The washer's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Washer                                                          |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, washed
/// | 1200 / 2000 energy                                              |
/// | Using 30 energy a tick                                          |  drawn last tick
/// | Washing                                                         |  status
/// | [####......]                                                    |  its water
/// | 750 / 1000 mB of Water                                          |
/// | Ore Rubble                                                      |
/// | [ ][ ][ ]                                                       |  to wash
/// +-----------------------------------------------------------------+
/// ```
fn screen(ui: &mut Ui) {
    ui.panel("panel", S::column().pad(7.0).gap(4.0), |ui| {
        ui.title();
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                crate::machine::power_bar(ui);
                crate::machine::meter_line(ui);
                ui.text_bind(|view, out| {
                    let wash = view.menu_component(&WASH).unwrap_or_default();
                    let status: &str = wash.status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.washer.{status}"), &args);
                });
                crate::machine::tank_bar(ui, WATER_CAPACITY);
            });
            ui.group("output");
        });
        ui.text("cogwork.washer.input");
        ui.group("input");
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.washer.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        assert!(crate::translated("cogwork.washer.input"), "its input");
    }

    #[test]
    fn what_a_washer_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Wash, paddle), 0, "its paddle");
        assert_eq!(offset_of!(Wash, spent), 8, "the energy spent");
        assert_eq!(offset_of!(Wash, status), 16, "its status");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let wash = Wash {
            status: 99,
            ..Wash::default()
        };
        assert_eq!(wash.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn it_holds_four_washes_of_water() {
        assert_eq!(WATER_CAPACITY / work::WATER_PER_WASH, 4, "a bucket");
    }

    #[test]
    fn the_washer_has_its_models() {
        for file in [
            "models/block/washer.json",
            "models/block/washer_water.json",
            "models/item/washer.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
