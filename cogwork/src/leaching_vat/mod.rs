//! The leaching vat: a plastic vat of sulfuric acid that leaches Rich Ore Rubble into Pure, the
//! last step of raising a grade (`rubble::Grade`), after flotation. Ore Rubble comes in by tube or
//! by hand, and each leach takes acid and engine energy (`work`).
//!
//! - Acid it draws itself out of a liquid tube or a tank on any side but its front, and keeps in
//!   its own state, which its item carries when it is broken, by hand or by a machine.
//! - Pure Ore Rubble leaves by its front, the side that faced the player who placed it, into a
//!   tube or a machine there. Ore Rubble of any other grade passes through untouched.
//!
//! Its shape is the Blockbench model `assets/models/block/leaching_vat.json`, a cream composite vat
//! in a steel frame, and `leaching_vat_acid.json` with the acid in it, bubbling, while it holds
//! some.

mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, ClientView, Component, ENERGY, Facings, Holder, Holds, ItemStack, Menu, Registry,
    S, Slots, Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::rubble::{ORE_RUBBLE, ore_of};

pub static LEACHING_VAT: Block = Block::named("leaching_vat");
static LEACHING_VAT_MENU: Menu = Menu::named("leaching_vat");

/// Millibuckets of acid it holds: twenty leaches.
pub const ACID_HELD: u32 = 2_000;
/// Its slots: Ore Rubble to leach, then what it leached.
const INPUT: u32 = 3;
const OUTPUT: u32 = 3;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 200;
const STORES: i64 = 4_000;

value! {
    /// A leaching vat's state: the energy `spent` on the Ore Rubble it leaches, which stays in its
    /// input until it is leached, its `status`, and millibuckets of `acid` it holds. Carried on its
    /// item when it is broken.
    #[derive(Default)]
    pub struct Leach {
        pub spent: i64,
        pub status: u32,
        pub acid: u32,
    }
}
pub static LEACH: Component<Leach> = Component::named("cogwork:leach");

/// What the leaching vat is doing or lacks; its screen says `cogwork.leaching_vat.<name>`. Kept in
/// `Leach::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoPower,
    Leaching,
    NoAcid,
    /// What it would leach, or pass through, has no room in its output.
    Full,
}

impl Leach {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &LEACH,
        Leach::default(),
        component_flags::SAVE | component_flags::SYNC_MENU | component_flags::CARRIED,
    );
    r.system(&LEACH, work::tick);
    r.block(&LEACHING_VAT)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("leaching_vat")
        .prop("acid", ["false", "true"])
        .states(|props, state| {
            if props.is("acid", "true") {
                state.model_file("leaching_vat_acid")
            } else {
                state
            }
        })
        .hitbox([0.0; 3], [16.0, 14.0, 16.0])
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&LEACH)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(INPUT + OUTPUT)
        })
        .opens(&LEACHING_VAT_MENU)
        .tab(&crate::TAB);
    r.menu(&LEACHING_VAT_MENU)
        .group(
            "input",
            Slots::row(INPUT).accepts(ORE_RUBBLE.name()).holder(),
        )
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
}

/// Puts `stack` from a tube into the leaching vat `at`: Ore Rubble into its input, nothing else.
/// None for what is no leaching vat.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &LEACH)?;
    if ore_of(&stack).is_none() {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("input")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The leaching vat's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Leaching Vat                                                    |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, leached
/// | 1200 / 4000 energy                                              |
/// | Using 50 energy a tick                                          |  drawn last tick
/// | Leaching                                                        |  status
/// | [####......]                                                    |  its acid
/// | 900 / 2000 mB of Sulfuric Acid                                  |
/// | Ore Rubble                                                      |
/// | [ ][ ][ ]                                                       |  to leach
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
                    let status: &str = leach(view).status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.leaching_vat.{status}"), &args);
                });
                let acid = "cogwork:sulfuric_acid";
                crate::machine::kept_bar(ui, ACID_HELD, acid, |view| leach(view).acid);
            });
            ui.group("output");
        });
        ui.text("cogwork.leaching_vat.input");
        ui.group("input");
        ui.player_inventory(true);
    });
}

/// The state of the leaching vat whose menu is open.
fn leach(view: &ClientView) -> Leach {
    view.menu_component(&LEACH).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.leaching_vat.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        assert!(crate::translated("cogwork.leaching_vat.input"), "its input");
    }

    #[test]
    fn what_a_leaching_vat_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Leach, spent), 0, "the energy spent");
        assert_eq!(offset_of!(Leach, status), 8, "its status");
        assert_eq!(offset_of!(Leach, acid), 12, "its acid");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let leach = Leach {
            status: 99,
            ..Leach::default()
        };
        assert_eq!(leach.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn the_leaching_vat_has_its_models() {
        for file in [
            "models/block/leaching_vat.json",
            "models/block/leaching_vat_acid.json",
            "models/item/leaching_vat.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
