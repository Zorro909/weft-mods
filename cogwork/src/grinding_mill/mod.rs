//! The grinding mill: the millstone's driven counterpart. Ore Rubble and sand come in by tube or
//! by hand, water by liquid tube (it is a holder of liquid that takes only water,
//! `liquid::Kind::Mill`), and with engine energy its runner grinds each Ore Rubble into its
//! metal's raw ore as the millstone does (`rubble::grinding`). The raw ore leaves by its front,
//! the side that faced the player who placed it, into a tube or a machine there; a timber tube
//! can pull it out too.
//!
//! Its shape is the Blockbench model `assets/models/block/grinding_mill.json`, a bed stone in a
//! low wooden vat under an open timber frame; the runner is
//! `assets/models/grinding_mill_runner.geo.json` (`runner`), its spindle rising to the top, where
//! a shaft from above meets it.

mod runner;
mod supply;
mod work;

use strum::{FromRepr, IntoStaticStr, VariantArray};
use weft_sdk::{
    Arg, Block, Component, ENERGY, Facings, Holder, Holds, ItemStack, Menu, Registry, S, Slots,
    Tag, Toward, Ui, World, component_flags, tool_tier, value,
};

use crate::liquid::HELD;
use crate::rubble::{ORE_RUBBLE, ore_of};

pub static GRINDING_MILL: Block = Block::named("grinding_mill");
static MILL_MENU: Menu = Menu::named("grinding_mill");
static SAND: Tag = Tag::named("sand");

/// Millibuckets of water it holds: eight grinds.
pub const WATER_CAPACITY: u32 = 1_000;
/// Its slots: Ore Rubble to grind, sand to grind with, then the raw ore.
const INPUT: u32 = 3;
const SAND_SLOTS: u32 = 1;
const OUTPUT: u32 = 3;
/// The most energy it takes from engines a tick, and the most it stores.
const MAX_IN: i64 = 200;
const STORES: i64 = 4_000;

value! {
    /// A grinding mill's state: its `runner` (an entity), the energy `spent` on the Ore Rubble
    /// it grinds, which stays in its input until it is ground, the grinds of sand it has left
    /// (`grit`), and its `status`.
    #[derive(Default)]
    pub struct Grinder {
        pub runner: u64,
        pub spent: i64,
        pub grit: u32,
        pub status: u32,
    }
}
static GRINDER: Component<Grinder> = Component::named("cogwork:grinder");

/// What the grinding mill is doing or lacks; its screen says `cogwork.grinding_mill.<name>`.
/// Kept in `Grinder::status` as its number, so the order is part of the save format: new ones
/// go last.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, FromRepr, IntoStaticStr, VariantArray)]
#[strum(serialize_all = "snake_case")]
#[repr(u32)]
pub enum Status {
    #[default]
    Idle,
    NoPower,
    Grinding,
    /// No loaded mod has a raw ore of the metal of its Ore Rubble.
    NoRawOre,
    NoWater,
    NoSand,
    /// The raw ore it would grind has no room in its output.
    Full,
}

impl Grinder {
    pub fn status(&self) -> Status {
        Status::from_repr(self.status).unwrap_or_default()
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status as u32;
    }
}

pub fn register(r: &mut Registry) {
    r.entity_component(
        &GRINDER,
        Grinder::default(),
        component_flags::SAVE | component_flags::SYNC_MENU,
    );
    r.system(&GRINDER, work::tick);
    r.block(&GRINDING_MILL)
        .facing(Facings::HORIZONTAL, Toward::Player)
        .model_file("grinding_mill")
        .hardness(3.5)
        .mined_by("pickaxe", tool_tier::HAND)
        .sound_group("base:block.stone")
        .entity(|entity| {
            entity
                .attach(&GRINDER)
                .attach(&HELD)
                .attach(&crate::owned::LOOPING)
                .attach_with(&ENERGY, crate::machine::energy(STORES, MAX_IN))
                .attach(&crate::machine::METER)
                .inventory(INPUT + SAND_SLOTS + OUTPUT)
        })
        .opens(&MILL_MENU)
        .tab(&crate::TAB);
    let sand = format!("#{}", SAND.name());
    r.menu(&MILL_MENU)
        .group(
            "input",
            Slots::row(INPUT).accepts(ORE_RUBBLE.name()).holder(),
        )
        .group("sand", Slots::row(SAND_SLOTS).accepts(&sand).holder())
        .group("output", Slots::row(OUTPUT).output().holder())
        .player_inventory()
        .screen(screen);
    runner::register(r);
}

/// Puts `stack` from a tube into the grinding mill `at`: Ore Rubble into its input, sand into
/// its sand slot, nothing else. None for what is no grinding mill.
pub fn insert(world: &World, at: Holder, stack: ItemStack, simulate: bool) -> Option<ItemStack> {
    world.component(at, &GRINDER)?;
    let group = if ore_of(&stack).is_some() {
        "input"
    } else if SAND.on_item(stack.item) {
        "sand"
    } else {
        return Some(stack);
    };
    let slots = world.inventory(at).find_group(group)?;
    Some(crate::store::insert_into(slots, stack, simulate))
}

/// The grinding mill's screen, above the player's inventory:
///
/// ```text
/// +-----------------------------------------------------------------+
/// | Grinding Mill                                                   |
/// | [#####.....]                              [ ][ ][ ]             |  stored energy, raw ore
/// | 2400 / 4000 energy                                              |
/// | Using 40 energy a tick                                          |  drawn last tick
/// | Grinding                                                        |  status
/// | [####......]                                                    |  its water
/// | 625 / 1000 mB of Water                                          |
/// | Ore Rubble              Sand                                    |
/// | [ ][ ][ ]               [ ]                                     |  to grind, to grind with
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
                    let grinder = view.menu_component(&GRINDER).unwrap_or_default();
                    let status: &str = grinder.status().into();
                    let args: [Arg; 0] = [];
                    out.set(&format!("cogwork.grinding_mill.{status}"), &args);
                });
                crate::machine::tank_bar(ui, WATER_CAPACITY);
            });
            ui.group("output");
        });
        ui.row(S::row().gap(10.0), |ui| {
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.grinding_mill.input");
                ui.group("input");
            });
            ui.column(S::column().gap(3.0), |ui| {
                ui.text("cogwork.grinding_mill.sand");
                ui.group("sand");
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
            let key = format!("cogwork.grinding_mill.{}", <&str>::from(status));
            assert!(crate::translated(&key), "{key} in every lang file");
        }
        for label in ["input", "sand"] {
            let key = format!("cogwork.grinding_mill.{label}");
            assert!(crate::translated(&key), "{key} in every lang file");
        }
    }

    #[test]
    fn what_a_grinder_keeps_is_saved_as_laid_out() {
        use std::mem::offset_of;
        assert_eq!(offset_of!(Grinder, runner), 0, "its runner");
        assert_eq!(offset_of!(Grinder, spent), 8, "the energy spent");
        assert_eq!(offset_of!(Grinder, grit), 16, "its sand");
        assert_eq!(offset_of!(Grinder, status), 20, "its status");
    }

    #[test]
    fn an_unknown_saved_status_reads_as_idle() {
        let grinder = Grinder {
            status: 99,
            ..Grinder::default()
        };
        assert_eq!(grinder.status(), Status::Idle, "from a newer cogwork");
    }

    #[test]
    fn it_holds_eight_grinds_of_water() {
        let grinds = WATER_CAPACITY / crate::rubble::grinding::WATER_PER_GRIND;
        assert_eq!(grinds, 8, "a bucket");
    }

    #[test]
    fn the_grinding_mill_has_its_model() {
        assert!(
            crate::has_asset("models/block/grinding_mill.json"),
            "no model"
        );
    }
}
