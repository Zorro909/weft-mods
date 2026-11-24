//! The polymerizer: a steel reactor that turns the refinery's gas into plastic pellets. Gas comes
//! in by liquid tube on any side (it is a holder of liquid that takes only gas), a catalyst by
//! tube or by hand, and with engine energy each batch of 250 mB of gas and one use of the
//! catalyst makes 4 pellets (`work`). The pellets leave by its front, into a tube or a machine
//! there.
//!
//! Its shape is the Blockbench model `assets/models/block/polymerizer.json`: a domed steel vessel
//! with a sight glass on a plinth, gas piped in at its side and a vent on top.

mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Facings, Holder, Holds, ItemStack, Menu, Registry, S, Slots,
    Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::liquid::HELD;
use crate::plastics::CATALYST;

pub static POLYMERIZER: Block = Block::named("polymerizer");
static POLYMERIZER_MENU: Menu = Menu::named("polymerizer");

/// Millibuckets of gas it holds: four batches.
pub const GAS_CAPACITY: u32 = 1_000;
/// Its slots: catalysts, the first in use, then the pellets made.
const CATALYSTS: u32 = 2;
const OUTPUT: u32 = 3;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 400;
const STORES: i64 = 4_000;

value! {
    /// A polymerizer's state: the energy `spent` on the batch under way, whose gas and catalyst
    /// stay in it until the batch is done, and its `status`.
    #[derive(Default)]
    pub struct Polymerizing {
        pub spent: i64,
        pub status: u32,
    }
}
static POLYMERIZING: Component<Polymerizing> = Component::named("cogwork:polymerizing");

/// What the polymerizer is doing or lacks; its screen says `cogwork.polymerizer.<name>`. Kept in
/// `Polymerizing::status` as its number, so the order is part of the save format: new ones go
/// last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    /// Waiting for a batch's gas.
    #[default]
    NoGas,
    NoCatalyst,
    NoPower,
    Working,
    /// A batch's pellets have no room in the output.
    Full,
}

impl Polymerizing {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &POLYMERIZING,
        Polymerizing::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&POLYMERIZING, work::tick);
    r.block(&POLYMERIZER)
        .grid_placement()
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("polymerizer")
        .hitbox([0.0; 3], [16.0; 3])
        .hardness(4.0)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&POLYMERIZING)
                .attach(&HELD)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(CATALYSTS + OUTPUT)
        })
        .opens(&POLYMERIZER_MENU)
        .tab(&crate::TAB);
    r.menu(&POLYMERIZER_MENU)
        .group(
            "catalyst",
            Slots::row(CATALYSTS).accepts(CATALYST.name()).holder(),
        )
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
}

/// Puts `stack` from a tube into the polymerizer `at`: a catalyst into its catalyst slots,
/// nothing else. None for what is no polymerizer.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &POLYMERIZING)?;
    if stack.item != CATALYST.id() {
        return Some(stack);
    }
    let slots = world.inventory(at).find_group("catalyst")?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The polymerizer's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Polymerizer                                                     |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, pellets
/// | 1200 / 4000 energy                                              |
/// | Using 200 energy a tick                                         |  drawn last tick
/// | Making plastic                                                  |  status
/// | [####......]                                                    |  its gas
/// | 750 / 1000 mB of Gas                                            |
/// | Catalyst                                                        |
/// | [ ][ ]                                                          |
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
                    let state = view.menu_component(&POLYMERIZING).unwrap_or_default();
                    let status: &str = state.status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.polymerizer.{status}"), &args);
                });
                crate::machine::tank_bar(ui, GAS_CAPACITY);
            });
            ui.group("output");
        });
        ui.text("cogwork.polymerizer.catalyst");
        ui.group("catalyst");
        ui.player_inventory(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_name() {
        for &status in Status::VARIANTS {
            let key = format!("cogwork.polymerizer.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        assert!(crate::translated("cogwork.polymerizer.catalyst"), "a label");
    }

    #[test]
    fn what_a_polymerizer_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Polymerizing, spent), 0, "the energy spent");
        assert_eq!(offset_of!(Polymerizing, status), 8, "its status");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_the_default() {
        let state = Polymerizing {
            status: 99,
            ..Polymerizing::default()
        };
        assert_eq!(state.status(), Status::NoGas, "from a newer cogwork");
    }

    #[test]
    fn it_holds_four_batches_of_gas() {
        assert_eq!(GAS_CAPACITY / work::GAS_A_BATCH, 4, "a bucket");
        assert!(
            crate::has_asset("models/block/polymerizer.json"),
            "its model"
        );
    }
}
