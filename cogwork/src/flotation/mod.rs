//! The flotation cell: a steel tank with a stirring rotor that floats Common Ore Rubble into Rich,
//! the second step of raising a grade (`rubble::Grade`), after the washer. Ore Rubble comes in by
//! tube or by hand, and each float takes water, creosote and engine energy, and leaves muddy water.
//!
//! - Water comes in by liquid tube on any side: it is a holder of liquid that takes only water,
//!   like the washer.
//! - Creosote it draws itself out of a liquid tube, a tank or a coke oven's drain beside it, on
//!   any side but its back, and keeps apart (`Float::creosote`).
//! - Muddy water it keeps apart too, and lets out of its back into a liquid tube or a clarifier
//!   there (`liquid::pour`).
//! - Rich Ore Rubble leaves by its front, the side that faced the player who placed it, into a
//!   tube or a machine there. Ore Rubble of any other grade passes through untouched.
//!
//! Its shape is the Blockbench model `assets/models/block/flotation_cell.json`, a riveted steel
//! tank with a froth lip at its front, and `flotation_cell_froth.json` with bubbling froth on top
//! while it floats; the rotor is `assets/models/flotation_rotor.geo.json` (`rotor`).

mod cell;
mod rotor;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Facings, Holder, Holds, ItemStack, Menu, Registry, S, Slots,
    Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::liquid::HELD;
use crate::rubble::{ORE_RUBBLE, ore_of};

pub static FLOTATION_CELL: Block = Block::named("flotation_cell");
static FLOTATION_MENU: Menu = Menu::named("flotation_cell");

/// Millibuckets of water it holds: four floats.
pub const WATER_CAPACITY: u32 = 2_000;
/// Its slots: Ore Rubble to float, then what it floated.
const INPUT: u32 = 3;
const OUTPUT: u32 = 3;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 200;
const STORES: i64 = 2_400;

value! {
    /// A flotation cell's state: its `rotor` (an entity), the energy `spent` on the Ore Rubble it
    /// floats, which stays in its input until it is floated, its `status`, and millibuckets of
    /// `creosote` and of `muddy` water it keeps beside the water it holds.
    #[derive(Default)]
    pub struct Float {
        pub rotor: u64,
        pub spent: i64,
        pub status: u32,
        pub creosote: u32,
        pub muddy: u32,
    }
}
static FLOAT: Component<Float> = Component::named("cogwork:float");

/// What the flotation cell is doing or lacks; its screen says `cogwork.flotation_cell.<name>`.
/// Kept in `Float::status` as its number, so the order is part of the save format: new ones go
/// last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoPower,
    Floating,
    NoWater,
    NoCreosote,
    /// Its muddy water has nowhere to go.
    Muddy,
    /// What it would float, or pass through, has no room in its output.
    Full,
}

impl Float {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &FLOAT,
        Float::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&FLOAT, work::tick);
    r.block(&FLOTATION_CELL)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("flotation_cell")
        .prop("froth", ["false", "true"])
        .states(|props, state| {
            if props.is("froth", "true") {
                state.model_file("flotation_cell_froth")
            } else {
                state
            }
        })
        .hardness(3.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&FLOAT)
                .attach(&HELD)
                .attach(&crate::owned::LOOPING)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(INPUT + OUTPUT)
        })
        .opens(&FLOTATION_MENU)
        .tab(&crate::TAB);
    r.menu(&FLOTATION_MENU)
        .group(
            "input",
            Slots::row(INPUT).accepts(ORE_RUBBLE.name()).holder(),
        )
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    rotor::register(r);
}

/// Puts `stack` from a tube into the flotation cell `at`: Ore Rubble into its input, nothing else.
/// None for what is no flotation cell.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &FLOAT)?;
    if ore_of(&stack).is_none() {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("input")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The flotation cell's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Flotation Cell                                                  |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, floated
/// | 1200 / 2400 energy                                              |
/// | Using 40 energy a tick                                          |  drawn last tick
/// | Floating                                                        |  status
/// | [####......]                                                    |  its water
/// | 1500 / 2000 mB of Water                                         |
/// | [######....]                                                    |  its creosote
/// | 300 / 500 mB of Creosote                                        |
/// | [##........]                                                    |  its muddy water
/// | 250 / 1000 mB of Muddy Water                                    |
/// | Ore Rubble                                                      |
/// | [ ][ ][ ]                                                       |  to float
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
                    let float = view.menu_component(&FLOAT).unwrap_or_default();
                    let status: &str = float.status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.flotation_cell.{status}"), &args);
                });
                crate::machine::tank_bar(ui, WATER_CAPACITY);
                let float =
                    |view: &weft_sdk::ClientView| view.menu_component(&FLOAT).unwrap_or_default();
                let (creosote, muddy) = ("cogwork:creosote", "cogwork:muddy_water");
                crate::machine::kept_bar(ui, cell::CREOSOTE_HELD, creosote, move |view| {
                    float(view).creosote
                });
                crate::machine::kept_bar(ui, cell::MUDDY_HELD, muddy, move |view| {
                    float(view).muddy
                });
            });
            ui.group("output");
        });
        ui.text("cogwork.flotation_cell.input");
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
            let key = format!("cogwork.flotation_cell.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        assert!(
            crate::translated("cogwork.flotation_cell.input"),
            "its input"
        );
    }

    #[test]
    fn what_a_flotation_cell_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Float, rotor), 0, "its rotor");
        assert_eq!(offset_of!(Float, spent), 8, "the energy spent");
        assert_eq!(offset_of!(Float, status), 16, "its status");
        assert_eq!(offset_of!(Float, creosote), 20, "its creosote");
        assert_eq!(offset_of!(Float, muddy), 24, "its muddy water");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let float = Float {
            status: 99,
            ..Float::default()
        };
        assert_eq!(float.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn the_flotation_cell_has_its_models() {
        for file in [
            "models/block/flotation_cell.json",
            "models/block/flotation_cell_froth.json",
            "models/item/flotation_cell.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
