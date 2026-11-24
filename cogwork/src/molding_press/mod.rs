//! The molding press: a hydraulic press that shapes plastic in a steel mold. Pellets, plastic
//! sheets and steel ingots come in by tube or by hand, a mold goes into its mold slot, and with
//! engine energy, 400 a press, it makes the mold's part (`molds`): a plastic sheet, gear or pipe
//! from pellets, or a composite panel from a sheet and a steel ingot. What it presses stays in the
//! input until its part is in the output, and the part leaves by its front.
//!
//! Its shape is the Blockbench model `assets/models/block/molding_press.json`, a steel bed and
//! two columns under a crosshead with its cylinder; the ram is an owned entity that comes down at
//! each press (`ram`).

mod molds;
mod ram;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Facings, Holder, Holds, ItemStack, Menu, Registry, S, Slots,
    Toward, Ui, World, component_flags, tool_tier, value,
};

pub static MOLDING_PRESS: Block = Block::named("molding_press");
static MOLDING_PRESS_MENU: Menu = Menu::named("molding_press");

/// Its slots: what it presses, the mold, then the parts it made.
const INPUT: u32 = 3;
const MOLD: u32 = 1;
const OUTPUT: u32 = 3;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 200;
const STORES: i64 = 2_000;

value! {
    /// A molding press's state: its `ram` (an entity), the energy `spent` on the press under way
    /// and its `status`.
    #[derive(Default)]
    pub struct Pressing {
        pub ram: u64,
        pub spent: i64,
        pub status: u32,
    }
}
static PRESSING: Component<Pressing> = Component::named("cogwork:pressing");

/// What the press is doing or lacks; its screen says `cogwork.molding_press.<name>`. Kept in
/// `Pressing::status` as its number, so the order is part of the save format: new ones go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// It has no mold in its mold slot.
    #[default]
    NoMold,
    /// Its input lacks what its mold presses.
    Idle,
    NoPower,
    Pressing,
    /// The part has no room in the output.
    Full,
}

impl Pressing {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &PRESSING,
        Pressing::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&PRESSING, work::tick);
    r.block(&MOLDING_PRESS)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("molding_press")
        .hitbox([0.0; 3], [16.0; 3])
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&PRESSING)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(INPUT + MOLD + OUTPUT)
        })
        .opens(&MOLDING_PRESS_MENU)
        .tab(&crate::TAB);
    let mold_rule = molds::mold_rule();
    r.menu(&MOLDING_PRESS_MENU)
        .group("input", Slots::row(INPUT).holder())
        .group("mold", Slots::single().accepts(&mold_rule).holder())
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    molds::register(r);
    ram::register(r);
}

/// Puts `stack` from a tube into the molding press `at`: a mold into its mold slot, what the
/// mold in it presses into its input, nothing else, so a part it made never comes back round.
/// None for what is no molding press.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &PRESSING)?;
    let inventory = world.inventory(at);
    let fitted = inventory
        .find_group("mold")
        .and_then(|mold| molds::shape_of(mold.get(0)));
    let group = if molds::shape_of(stack).is_some() {
        "mold"
    } else if fitted.is_some_and(|shape| shape.presses(stack.item)) {
        "input"
    } else {
        return Some(stack);
    };
    let slots = inventory.find_group(group)?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The molding press's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Molding Press                                                   |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, parts
/// | 1200 / 2000 energy                                              |
/// | Using 40 energy a tick                                          |  drawn last tick
/// | Pressing                                                        |  status
/// | To press              Mold                                      |
/// | [ ][ ][ ]             [ ]                                       |
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
                    let state = view.menu_component(&PRESSING).unwrap_or_default();
                    let status: &str = state.status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.molding_press.{status}"), &args);
                });
            });
            ui.group("output");
        });
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.molding_press.input");
                ui.group("input");
            });
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.molding_press.mold");
                ui.group("mold");
            });
        });
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.molding_press.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        for label in ["input", "mold"] {
            let key = format!("cogwork.molding_press.{label}");
            assert!(crate::translated(&key), "{key}");
        }
    }

    #[test]
    fn what_a_press_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Pressing, ram), 0, "its ram");
        assert_eq!(offset_of!(Pressing, spent), 8, "the energy spent");
        assert_eq!(offset_of!(Pressing, status), 16, "its status");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_the_default() {
        let state = Pressing {
            status: 99,
            ..Pressing::default()
        };
        assert_eq!(state.status(), Status::NoMold, "from a newer cogwork");
    }

    #[test]
    fn the_press_has_its_models() {
        for file in [
            "models/block/molding_press.json",
            "models/item/molding_press.json",
        ] {
            assert!(crate::has_asset(file), "no {file}");
        }
    }
}
